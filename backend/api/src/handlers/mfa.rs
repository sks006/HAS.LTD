// backend/api/src/handlers/mfa.rs
//! Phase 6B: Multi-Factor Authentication (TOTP & Recovery) Handlers
//! Endpoints:
//! - POST /auth/mfa/totp/enroll
//! - POST /auth/mfa/totp/verify
//! - POST /auth/mfa/totp/disable
//! - POST /auth/login/totp
//! - POST /auth/login/recovery

use axum::{
    extract::State,
    http::HeaderMap,
    response::Response,
    Json,
};
use redis::{AsyncCommands, Script};
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dtos::auth_dto::{
        DisableMfaRequest, EnrollMfaRequest, EnrollMfaResponse, LoginRecoveryRequest,
        LoginTotpRequest, SuccessResponse, VerifyEnrollmentRequest,
    },
    errors::ApiError,
    handlers::auth::{
        extract_client_ip, extract_user_agent, issue_login_session, record_audit_event,
    },
    metrics::METRICS,
    middleware::auth_guard::{AuthUser, MfaEnrollAuthUser},
    state::AppState,
};
use domain::models::{
    credentials::MfaCredential,
    identifiers::MfaCredentialId,
    user::{User, UserStatus},
};

/// Helper to check rate limits using Redis atomic script.
async fn check_rate_limit(
    redis: &mut redis::aio::ConnectionManager,
    key: &str,
    limit: i64,
    window_secs: u64,
) -> Result<(), ApiError> {
    let script = Script::new(
        r#"
        local current = redis.call('INCR', KEYS[1])
        if tonumber(current) == 1 then
            redis.call('EXPIRE', KEYS[1], ARGV[2])
        end
        return current
        "#,
    );
    let attempts: i64 = script
        .key(key)
        .arg(limit)
        .arg(window_secs)
        .invoke_async(redis)
        .await
        .unwrap_or(1);

    if attempts > limit {
        return Err(ApiError::RateLimited {
            retry_after: window_secs,
        });
    }
    Ok(())
}

/// Handler 1: POST /auth/mfa/totp/enroll
/// Initiates MFA enrollment by generating a TOTP secret and 10 recovery codes.
pub async fn mfa_enroll(
    auth_user: MfaEnrollAuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<EnrollMfaRequest>,
) -> Result<Json<EnrollMfaResponse>, ApiError> {
    payload.validate().map_err(|e| ApiError::BadRequestWithCode {
        code: "validation_error".to_string(),
        message: e.to_string(),
    })?;

    let user_id = auth_user.user_id.into_inner();
    let ip = extract_client_ip(&headers);
    let user_agent = extract_user_agent(&headers);

    // Rate limiting: 5 attempts per 15 minutes
    if let Some(mut redis) = state.redis.clone() {
        let rl_key = format!("rl:mfa_enroll:{}", user_id);
        check_rate_limit(&mut redis, &rl_key, 5, 900).await?;
    }

    // Verify current password via Argon2Gate (FIX-H4)
    let cred_row: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT secret_hash FROM credentials
        WHERE user_id = $1 AND kind = 'password' AND provider = 'local'
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    let (current_secret_hash,) = match cred_row {
        Some(r) => r,
        None => {
            return Err(ApiError::UnauthorizedWithCode {
                code: "invalid_credentials".to_string(),
                message: "Credentials row not found".to_string(),
            });
        }
    };

    let cur_pw = payload.current_password.clone();
    let is_valid = state
        .argon2_gate()
        .run(move || crypto::password::verify_password(&cur_pw, &current_secret_hash))
        .await
        .map_err(|e| ApiError::ServiceUnavailable(e.to_string()))?
        .map_err(|e| ApiError::Internal(format!("Password verification failed: {:?}", e)))?;

    if !is_valid {
        return Err(ApiError::UnauthorizedWithCode {
            code: "invalid_credentials".to_string(),
            message: "Current password is incorrect".to_string(),
        });
    }

    // [REQUIRED-1] Check if MFA is already confirmed for this user.
    // If a confirmed row exists, reject with 409 Conflict mfa_already_enrolled.
    let existing_confirmed: Option<(i32,)> = sqlx::query_as(
        r#"
        SELECT 1 FROM mfa_credentials
        WHERE user_id = $1 AND confirmed_at IS NOT NULL
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    if existing_confirmed.is_some() {
        return Err(ApiError::ConflictWithCode {
            code: "mfa_already_enrolled".to_string(),
            message: "MFA is already enabled. Disable it before re-enrolling.".to_string(),
        });
    }

    // Generate fresh TOTP secret and otpauth URL
    let (secret_bytes, otpauth_url) = crypto::totp::generate_totp_secret();
    let secret_str = otpauth_url
        .split("secret=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .unwrap_or("")
        .to_string();

    // Encrypt secret with AES-256-GCM using master key from totp.key
    let secret_encrypted = crypto::totp::encrypt_totp_secret(&secret_bytes, &state.totp_key)
        .map_err(|e| ApiError::Internal(format!("Failed to encrypt TOTP secret: {:?}", e)))?;

    // Generate 10 recovery codes and hash them with Argon2id
    let (plaintext_codes, hash_codes) = crypto::totp::generate_recovery_codes(10)
        .map_err(|e| ApiError::Internal(format!("Failed to generate recovery codes: {:?}", e)))?;

    // Upsert into mfa_credentials with confirmed_at = NULL (half-enrolled state)
    let mfa_id = MfaCredentialId::now_v7();
    sqlx::query(
        r#"
        INSERT INTO mfa_credentials (id, user_id, kind, secret_encrypted, recovery_codes_hash, confirmed_at, created_at)
        VALUES ($1, $2, 'totp', $3, $4, NULL, now())
        ON CONFLICT (user_id) DO UPDATE SET
            secret_encrypted = EXCLUDED.secret_encrypted,
            recovery_codes_hash = EXCLUDED.recovery_codes_hash,
            confirmed_at = NULL,
            created_at = now()
        "#,
    )
    .bind(mfa_id.into_inner())
    .bind(user_id)
    .bind(&secret_encrypted)
    .bind(&hash_codes)
    .execute(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to persist mfa credentials: {:?}", e)))?;

    record_audit_event(
        &state,
        Some(user_id),
        "auth.mfa.enrolled",
        ip,
        &user_agent,
        json!({ "user_id": user_id }),
    )
    .await;

    Ok(Json(EnrollMfaResponse {
        otpauth_url,
        secret: secret_str,
        recovery_codes: plaintext_codes,
    }))
}

/// Handler 2: POST /auth/mfa/totp/verify
/// Confirms and activates MFA enrollment using the first valid TOTP code.
pub async fn mfa_verify(
    auth_user: MfaEnrollAuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<VerifyEnrollmentRequest>,
) -> Result<Json<SuccessResponse>, ApiError> {
    payload.validate().map_err(|e| ApiError::BadRequestWithCode {
        code: "validation_error".to_string(),
        message: e.to_string(),
    })?;

    let user_id = auth_user.user_id.into_inner();
    let ip = extract_client_ip(&headers);
    let user_agent = extract_user_agent(&headers);

    // Rate limiting: 5 attempts per 10 minutes
    if let Some(mut redis) = state.redis.clone() {
        let rl_key = format!("rl:mfa_verify:{}", user_id);
        check_rate_limit(&mut redis, &rl_key, 5, 600).await?;
    }

    // Lookup pending (unconfirmed) enrollment row
    let cred: Option<MfaCredential> = sqlx::query_as(
        r#"
        SELECT id, user_id, kind, secret_encrypted, recovery_codes_hash, confirmed_at, created_at
        FROM mfa_credentials
        WHERE user_id = $1 AND confirmed_at IS NULL
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    let cred = match cred {
        Some(c) => c,
        None => {
            return Err(ApiError::NotFoundWithCode {
                code: "not_found".to_string(),
                message: "No pending MFA enrollment found".to_string(),
            });
        }
    };

    // Check enrollment expiration (30 minutes)
    if chrono::Utc::now() - cred.created_at > chrono::Duration::minutes(30) {
        let _ = sqlx::query("DELETE FROM mfa_credentials WHERE id = $1")
            .bind(cred.id.into_inner())
            .execute(&state.db_pool)
            .await;

        return Err(ApiError::NotFoundWithCode {
            code: "enrollment_expired".to_string(),
            message: "MFA enrollment has expired. Please initiate enrollment again.".to_string(),
        });
    }

    // Decrypt secret and verify submitted TOTP code
    let secret = crypto::totp::decrypt_totp_secret(&cred.secret_encrypted, &state.totp_key)
        .map_err(|e| ApiError::Internal(format!("Decryption failure: {:?}", e)))?;

    let is_valid = crypto::totp::verify_totp(&secret, &payload.code)
        .map_err(|e| ApiError::Internal(format!("TOTP verification failed: {:?}", e)))?;

    if !is_valid {
        return Err(ApiError::UnauthorizedWithCode {
            code: "mfa_invalid".to_string(),
            message: "Invalid TOTP code".to_string(),
        });
    }

    // Confirm enrollment
    sqlx::query(
        r#"
        UPDATE mfa_credentials
        SET confirmed_at = now()
        WHERE id = $1
        "#,
    )
    .bind(cred.id.into_inner())
    .execute(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to confirm MFA enrollment: {:?}", e)))?;

    record_audit_event(
        &state,
        Some(user_id),
        "auth.mfa.enrollment_confirmed",
        ip,
        &user_agent,
        json!({ "user_id": user_id }),
    )
    .await;

    Ok(Json(SuccessResponse { success: true }))
}

/// Handler 3: POST /auth/mfa/totp/disable
/// Disables MFA by deleting mfa_credentials upon password and TOTP/recovery verification.
pub async fn mfa_disable(
    auth_user: AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DisableMfaRequest>,
) -> Result<Json<SuccessResponse>, ApiError> {
    payload.validate().map_err(|e| ApiError::BadRequestWithCode {
        code: "validation_error".to_string(),
        message: e.to_string(),
    })?;

    let user_id = auth_user.user_id.into_inner();
    let ip = extract_client_ip(&headers);
    let user_agent = extract_user_agent(&headers);

    // Rate limiting: 3 attempts per 15 minutes
    if let Some(mut redis) = state.redis.clone() {
        let rl_key = format!("rl:mfa_disable:{}", user_id);
        check_rate_limit(&mut redis, &rl_key, 3, 900).await?;
    }

    // Verify current password via Argon2Gate
    let cred_row: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT secret_hash FROM credentials
        WHERE user_id = $1 AND kind = 'password' AND provider = 'local'
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    let (current_secret_hash,) = match cred_row {
        Some(r) => r,
        None => {
            return Err(ApiError::UnauthorizedWithCode {
                code: "invalid_credentials".to_string(),
                message: "Credentials row not found".to_string(),
            });
        }
    };

    let cur_pw = payload.current_password.clone();
    let is_pw_valid = state
        .argon2_gate()
        .run(move || crypto::password::verify_password(&cur_pw, &current_secret_hash))
        .await
        .map_err(|e| ApiError::ServiceUnavailable(e.to_string()))?
        .map_err(|e| ApiError::Internal(format!("Password verification failed: {:?}", e)))?;

    if !is_pw_valid {
        return Err(ApiError::UnauthorizedWithCode {
            code: "invalid_credentials".to_string(),
            message: "Current password is incorrect".to_string(),
        });
    }

    // Query active confirmed MFA credentials
    let mfa_cred: Option<MfaCredential> = sqlx::query_as(
        r#"
        SELECT id, user_id, kind, secret_encrypted, recovery_codes_hash, confirmed_at, created_at
        FROM mfa_credentials
        WHERE user_id = $1 AND confirmed_at IS NOT NULL
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    let mfa_cred = match mfa_cred {
        Some(c) => c,
        None => {
            return Err(ApiError::BadRequestWithCode {
                code: "mfa_not_enabled".to_string(),
                message: "MFA is not enabled".to_string(),
            });
        }
    };

    // Check submitted code against TOTP or recovery codes
    let mut code_valid = false;
    let mut method = "totp";

    if payload.code.len() == 6 {
        if let Ok(secret) = crypto::totp::decrypt_totp_secret(&mfa_cred.secret_encrypted, &state.totp_key) {
            if let Ok(true) = crypto::totp::verify_totp(&secret, &payload.code) {
                code_valid = true;
            }
        }
    }

    if !code_valid {
        // Fallback: check recovery codes
        if let Ok(true) = crypto::totp::verify_recovery_code(&payload.code, &mfa_cred.recovery_codes_hash) {
            code_valid = true;
            method = "recovery";
        }
    }

    if !code_valid {
        return Err(ApiError::UnauthorizedWithCode {
            code: "mfa_invalid".to_string(),
            message: "Invalid MFA code or recovery code".to_string(),
        });
    }

    // Delete mfa_credentials row
    sqlx::query("DELETE FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to delete MFA credentials: {:?}", e)))?;

    record_audit_event(
        &state,
        Some(user_id),
        "auth.mfa.disabled",
        ip,
        &user_agent,
        json!({ "user_id": user_id, "method": method }),
    )
    .await;

    Ok(Json(SuccessResponse { success: true }))
}

/// Handler 4: POST /auth/login/totp
/// Completes MFA login challenge using a single-use mfa_token and 6-digit TOTP code.
pub async fn login_totp(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LoginTotpRequest>,
) -> Result<Response, ApiError> {
    METRICS.counter_inc_labeled("auth_mfa_attempts_total", "method", "totp");
    match login_totp_inner(state, headers, payload).await {
        Ok(res) => Ok(res),
        Err(err) => {
            METRICS.counter_inc_labeled("auth_mfa_failures_total", "method", "totp");
            Err(err)
        }
    }
}

async fn login_totp_inner(
    state: AppState,
    headers: HeaderMap,
    payload: LoginTotpRequest,
) -> Result<Response, ApiError> {
    payload.validate().map_err(|e| ApiError::BadRequestWithCode {
        code: "validation_error".to_string(),
        message: e.to_string(),
    })?;

    let ip = extract_client_ip(&headers);
    let user_agent = extract_user_agent(&headers);

    // Verify mfa_token signature and expiration
    let claims = match state.jwt_keys.verify_access_token(&payload.mfa_token) {
        Ok(c) => c,
        Err(crypto::errors::CryptoError::Jwt(e)) => match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature
            | jsonwebtoken::errors::ErrorKind::ImmatureSignature => {
                return Err(ApiError::UnauthorizedWithCode {
                    code: "mfa_challenge_expired".to_string(),
                    message: "MFA challenge has expired".to_string(),
                });
            }
            _ => {
                return Err(ApiError::UnauthorizedWithCode {
                    code: "invalid_token".to_string(),
                    message: "Invalid token".to_string(),
                });
            }
        },
        Err(_) => {
            return Err(ApiError::UnauthorizedWithCode {
                code: "invalid_token".to_string(),
                message: "Invalid token".to_string(),
            });
        }
    };

    // Enforce scope == "mfa_pending"
    if claims.scope != "mfa_pending" {
        return Err(ApiError::UnauthorizedWithCode {
            code: "invalid_token".to_string(),
            message: "Token scope must be 'mfa_pending'".to_string(),
        });
    }

    // Atomic single-use GET + DEL from Redis nonce
    let challenge_key = format!("mfa:challenge:{}", claims.jti);
    let user_id_res: Option<String> = if let Some(mut redis) = state.redis.clone() {
        let script = Script::new(
            r#"
            local val = redis.call('GET', KEYS[1])
            if val then
                redis.call('DEL', KEYS[1])
                return val
            else
                return nil
            end
            "#,
        );
        script
            .key(&challenge_key)
            .invoke_async(&mut redis)
            .await
            .unwrap_or(None)
    } else {
        None
    };

    let stored_user_id = match user_id_res {
        Some(uid) => uid,
        None => {
            return Err(ApiError::UnauthorizedWithCode {
                code: "mfa_challenge_expired".to_string(),
                message: "MFA challenge expired or already consumed".to_string(),
            });
        }
    };

    if stored_user_id != claims.sub {
        return Err(ApiError::UnauthorizedWithCode {
            code: "mfa_challenge_expired".to_string(),
            message: "MFA challenge mismatch".to_string(),
        });
    }

    let user_uuid = Uuid::parse_str(&claims.sub).map_err(|_| ApiError::UnauthorizedWithCode {
        code: "invalid_token".to_string(),
        message: "Invalid user id in token".to_string(),
    })?;

    // Rate limiting: 5 attempts per 1 minute per user
    if let Some(mut redis) = state.redis.clone() {
        let rl_key = format!("rl:mfa_login:{}", user_uuid);
        check_rate_limit(&mut redis, &rl_key, 5, 60).await?;
    }

    // Load user from DB
    let user: Option<User> = sqlx::query_as(
        r#"
        SELECT id, email, email_normalized, name, role, status,
               email_verified_at, auth_version, created_at, updated_at, metadata
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_uuid)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    let user = match user {
        Some(u) => u,
        None => {
            return Err(ApiError::UnauthorizedWithCode {
                code: "invalid_credentials".to_string(),
                message: "User not found".to_string(),
            });
        }
    };

    if user.status != UserStatus::Active {
        return Err(ApiError::ForbiddenWithCode {
            code: "account_suspended".to_string(),
            message: "Account is not active".to_string(),
        });
    }

    // Load confirmed MFA credentials
    let mfa_cred: Option<MfaCredential> = sqlx::query_as(
        r#"
        SELECT id, user_id, kind, secret_encrypted, recovery_codes_hash, confirmed_at, created_at
        FROM mfa_credentials
        WHERE user_id = $1 AND confirmed_at IS NOT NULL
        "#,
    )
    .bind(user_uuid)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    let mfa_cred = match mfa_cred {
        Some(c) => c,
        None => {
            return Err(ApiError::UnauthorizedWithCode {
                code: "mfa_challenge_expired".to_string(),
                message: "No active MFA credentials".to_string(),
            });
        }
    };

    // Decrypt and verify TOTP code
    let secret = crypto::totp::decrypt_totp_secret(&mfa_cred.secret_encrypted, &state.totp_key)
        .map_err(|e| ApiError::Internal(format!("Decryption failure: {:?}", e)))?;

    let is_valid = crypto::totp::verify_totp(&secret, &payload.code)
        .map_err(|e| ApiError::Internal(format!("TOTP verification failed: {:?}", e)))?;

    if !is_valid {
        record_audit_event(
            &state,
            Some(user_uuid),
            "auth.mfa.login_failed",
            ip,
            &user_agent,
            json!({ "user_id": user_uuid.to_string(), "method": "totp" }),
        )
        .await;

        return Err(ApiError::UnauthorizedWithCode {
            code: "mfa_invalid".to_string(),
            message: "Invalid TOTP code".to_string(),
        });
    }

    // Reset email rate limit counter on successful MFA login
    if let Some(mut redis) = state.redis.clone() {
        let email_key = format!("rl:login:email:{}", user.email_normalized);
        let _: Result<(), _> = redis.del(&email_key).await;
    }

    // Issue session and tokens
    issue_login_session(
        &state,
        &user,
        ip,
        user_agent,
        "auth.mfa.login_success",
        json!({ "method": "totp" }),
    )
    .await
}

/// Handler 5: POST /auth/login/recovery
/// Completes MFA login challenge using a single-use mfa_token and a single-use recovery code.
pub async fn login_recovery(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LoginRecoveryRequest>,
) -> Result<Response, ApiError> {
    METRICS.counter_inc_labeled("auth_mfa_attempts_total", "method", "recovery");
    match login_recovery_inner(state, headers, payload).await {
        Ok(res) => Ok(res),
        Err(err) => {
            METRICS.counter_inc_labeled("auth_mfa_failures_total", "method", "recovery");
            Err(err)
        }
    }
}

async fn login_recovery_inner(
    state: AppState,
    headers: HeaderMap,
    payload: LoginRecoveryRequest,
) -> Result<Response, ApiError> {
    payload.validate().map_err(|e| ApiError::BadRequestWithCode {
        code: "validation_error".to_string(),
        message: e.to_string(),
    })?;

    let ip = extract_client_ip(&headers);
    let user_agent = extract_user_agent(&headers);

    // Verify mfa_token signature and expiration
    let claims = match state.jwt_keys.verify_access_token(&payload.mfa_token) {
        Ok(c) => c,
        Err(crypto::errors::CryptoError::Jwt(e)) => match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature
            | jsonwebtoken::errors::ErrorKind::ImmatureSignature => {
                return Err(ApiError::UnauthorizedWithCode {
                    code: "mfa_challenge_expired".to_string(),
                    message: "MFA challenge has expired".to_string(),
                });
            }
            _ => {
                return Err(ApiError::UnauthorizedWithCode {
                    code: "invalid_token".to_string(),
                    message: "Invalid token".to_string(),
                });
            }
        },
        Err(_) => {
            return Err(ApiError::UnauthorizedWithCode {
                code: "invalid_token".to_string(),
                message: "Invalid token".to_string(),
            });
        }
    };

    // Enforce scope == "mfa_pending"
    if claims.scope != "mfa_pending" {
        return Err(ApiError::UnauthorizedWithCode {
            code: "invalid_token".to_string(),
            message: "Token scope must be 'mfa_pending'".to_string(),
        });
    }

    // Atomic single-use GET + DEL from Redis nonce
    let challenge_key = format!("mfa:challenge:{}", claims.jti);
    let user_id_res: Option<String> = if let Some(mut redis) = state.redis.clone() {
        let script = Script::new(
            r#"
            local val = redis.call('GET', KEYS[1])
            if val then
                redis.call('DEL', KEYS[1])
                return val
            else
                return nil
            end
            "#,
        );
        script
            .key(&challenge_key)
            .invoke_async(&mut redis)
            .await
            .unwrap_or(None)
    } else {
        None
    };

    let stored_user_id = match user_id_res {
        Some(uid) => uid,
        None => {
            return Err(ApiError::UnauthorizedWithCode {
                code: "mfa_challenge_expired".to_string(),
                message: "MFA challenge expired or already consumed".to_string(),
            });
        }
    };

    if stored_user_id != claims.sub {
        return Err(ApiError::UnauthorizedWithCode {
            code: "mfa_challenge_expired".to_string(),
            message: "MFA challenge mismatch".to_string(),
        });
    }

    let user_uuid = Uuid::parse_str(&claims.sub).map_err(|_| ApiError::UnauthorizedWithCode {
        code: "invalid_token".to_string(),
        message: "Invalid user id in token".to_string(),
    })?;

    // Rate limiting: 5 attempts per 5 minutes per user
    if let Some(mut redis) = state.redis.clone() {
        let rl_key = format!("rl:mfa_recovery:{}", user_uuid);
        check_rate_limit(&mut redis, &rl_key, 5, 300).await?;
    }

    // Load user from DB
    let user: Option<User> = sqlx::query_as(
        r#"
        SELECT id, email, email_normalized, name, role, status,
               email_verified_at, auth_version, created_at, updated_at, metadata
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_uuid)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    let user = match user {
        Some(u) => u,
        None => {
            return Err(ApiError::UnauthorizedWithCode {
                code: "invalid_credentials".to_string(),
                message: "User not found".to_string(),
            });
        }
    };

    if user.status != UserStatus::Active {
        return Err(ApiError::ForbiddenWithCode {
            code: "account_suspended".to_string(),
            message: "Account is not active".to_string(),
        });
    }

    // [REQUIRED-2] Atomic recovery code array mutation with FOR UPDATE
    let mut tx = state
        .db_pool
        .begin()
        .await
        .map_err(|e| ApiError::Internal(format!("Transaction begin error: {:?}", e)))?;

    // Load confirmed MFA credentials with row lock
    let mfa_cred: Option<MfaCredential> = sqlx::query_as(
        r#"
        SELECT id, user_id, kind, secret_encrypted, recovery_codes_hash, confirmed_at, created_at
        FROM mfa_credentials
        WHERE user_id = $1 AND confirmed_at IS NOT NULL
        FOR UPDATE
        "#,
    )
    .bind(user_uuid)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    let mfa_cred = match mfa_cred {
        Some(c) => c,
        None => {
            return Err(ApiError::UnauthorizedWithCode {
                code: "mfa_challenge_expired".to_string(),
                message: "No active MFA credentials".to_string(),
            });
        }
    };

    // Verify recovery code against stored Argon2id hashes
    let match_idx = crypto::totp::verify_recovery_code_index(
        &payload.recovery_code,
        &mfa_cred.recovery_codes_hash,
    )
    .map_err(|e| ApiError::Internal(format!("Recovery code verification failed: {:?}", e)))?;

    let idx = match match_idx {
        Some(i) => i,
        None => {
            record_audit_event(
                &state,
                Some(user_uuid),
                "auth.mfa.recovery_failed",
                ip,
                &user_agent,
                json!({ "user_id": user_uuid.to_string(), "method": "recovery" }),
            )
            .await;

            return Err(ApiError::UnauthorizedWithCode {
                code: "mfa_invalid".to_string(),
                message: "Invalid recovery code".to_string(),
            });
        }
    };

    // Atomically consume recovery code by removing hash from array in DB
    let mut updated_hashes = mfa_cred.recovery_codes_hash.clone();
    updated_hashes.remove(idx);
    let remaining_codes = updated_hashes.len();

    sqlx::query(
        r#"
        UPDATE mfa_credentials
        SET recovery_codes_hash = $1
        WHERE id = $2
        "#,
    )
    .bind(&updated_hashes)
    .bind(mfa_cred.id.into_inner())
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to update recovery codes: {:?}", e)))?;

    tx.commit()
        .await
        .map_err(|e| ApiError::Internal(format!("Transaction commit failed: {:?}", e)))?;

    // Reset email rate limit counter on successful recovery login
    if let Some(mut redis) = state.redis.clone() {
        let email_key = format!("rl:login:email:{}", user.email_normalized);
        let _: Result<(), _> = redis.del(&email_key).await;
    }

    // Issue session and tokens
    issue_login_session(
        &state,
        &user,
        ip,
        user_agent,
        "auth.mfa.recovery_used",
        json!({ "method": "recovery", "remaining_codes": remaining_codes }),
    )
    .await
}
