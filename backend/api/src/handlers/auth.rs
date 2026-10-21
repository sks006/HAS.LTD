// backend/api/src/handlers/auth.rs

use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use redis::{AsyncCommands, Script};
use validator::Validate;
use uuid::Uuid;

use domain::models::identifiers::{AuditLogId, RefreshTokenId, SessionId};
use domain::models::user::{User, UserRole, UserStatus};
use domain::ports::email_port::{EmailMessage, EmailPort};
use crate::dtos::*;
use crate::errors::ApiError;
use crate::middleware::auth_guard::AuthUser;
use crate::state::AppState;

fn extract_email_domain(email: &str) -> String {
    email.split('@').nth(1).unwrap_or("unknown").to_string()
}

/// Normalizes and aggregates IP addresses: IPv4 is kept as-is; IPv6 has its lower 64 bits zeroed per [FIX-H3].
fn mask_ip(ip: &IpAddr) -> String {
    match ip {
        IpAddr::V4(v4) => v4.to_string(),
        IpAddr::V6(v6) => {
            let octets = v6.octets();
            let masked = std::net::Ipv6Addr::new(
                u16::from_be_bytes([octets[0], octets[1]]),
                u16::from_be_bytes([octets[2], octets[3]]),
                u16::from_be_bytes([octets[4], octets[5]]),
                u16::from_be_bytes([octets[6], octets[7]]),
                0, 0, 0, 0,
            );
            format!("{}/64", masked)
        }
    }
}

pub(crate) fn extract_client_ip(headers: &HeaderMap) -> IpAddr {
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(s) = forwarded.to_str() {
            if let Some(first_ip) = s.split(',').next() {
                if let Ok(ip) = IpAddr::from_str(first_ip.trim()) {
                    return ip;
                }
            }
        }
    }
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(s) = real_ip.to_str() {
            if let Ok(ip) = IpAddr::from_str(s.trim()) {
                return ip;
            }
        }
    }
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
}

pub(crate) fn extract_user_agent(headers: &HeaderMap) -> String {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

/// Helper to write audit log events directly via state.db_pool.
/// Never called inside a transaction. Metadata must never include secrets, passwords, OTPs, or tokens.
pub(crate) async fn record_audit_event(
    state: &AppState,
    user_id: Option<Uuid>,
    event: &str,
    ip: IpAddr,
    user_agent: &str,
    metadata: serde_json::Value,
) {
    crate::metrics::METRICS.counter_inc_labeled("auth_audit_events_total", "event", event);
    let audit_id = AuditLogId::now_v7();
    let now = chrono::Utc::now();
    let _ = sqlx::query(
        r#"
        INSERT INTO audit_log (id, user_id, event, ip_address, user_agent, metadata, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(audit_id.into_inner())
    .bind(user_id)
    .bind(event)
    .bind(ip)
    .bind(user_agent)
    .bind(metadata)
    .bind(now)
    .execute(&state.db_pool)
    .await;
}

async fn record_login_failure(
    state: &AppState,
    user_id: Option<Uuid>,
    event: &str,
    ip: IpAddr,
    user_agent: &str,
    metadata: serde_json::Value,
) {
    let reason = match event {
        "auth.login.failed_credentials" => "invalid_credentials",
        "auth.login.failed_suspended" => "account_suspended",
        "auth.login.failed_mfa_required" => "mfa_required",
        _ => "other",
    };
    crate::metrics::METRICS.counter_inc_labeled("auth_login_failures_total", "reason", reason);
    record_audit_event(state, user_id, event, ip, user_agent, metadata).await;
}

/// Handler 1: POST /auth/login
/// Implements Phase 3 Flow B authentication:
/// - Rate limiting via Redis Lua script for `rl:login:ip` and `rl:login:email`
/// - Password verification through Argon2Gate concurrency wrapper
/// - Temporary Phase 4 branch: Moderator/Admin returns HTTP 501 `mfa_not_yet_available`
/// - Single transaction creates session, refresh token (RTR), and audit log
/// - Signs RS256 access token with JwtKeys
/// - Sets `__Secure-has_refresh` cookie per Phase 3 §1.l
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Result<Response, ApiError> {
    crate::metrics::METRICS.counter_inc("auth_login_attempts_total");

    // 1. Validate payload
    payload.validate().map_err(|e| {
        crate::metrics::METRICS.counter_inc_labeled("auth_login_failures_total", "reason", "validation_error");
        ApiError::BadRequestWithCode {
            code: "bad_request".to_string(),
            message: e.to_string(),
        }
    })?;

    let client_ip = extract_client_ip(&headers);
    let masked_ip = mask_ip(&client_ip);
    let user_agent = extract_user_agent(&headers);
    let email_norm = payload.email.trim().to_ascii_lowercase();
    let email_domain = email_norm.split('@').nth(1).unwrap_or("unknown").to_string();

    // 2. IP Rate Limiting Lua script (10 attempts / 60 seconds)
    let ip_limit_script = Script::new(
        r#"
        local current = redis.call('INCR', KEYS[1])
        if tonumber(current) == 1 then
            redis.call('EXPIRE', KEYS[1], ARGV[2])
        end
        if tonumber(current) > tonumber(ARGV[1]) then
            local ttl = redis.call('TTL', KEYS[1])
            return {0, ttl, current}
        else
            return {1, 0, current}
        end
        "#,
    );

    let ip_key = format!("rl:login:ip:{}", masked_ip);
    let email_key = format!("rl:login:email:{}", email_norm);

    if let Some(mut redis) = state.redis.clone() {
        // Execute IP rate limit script
        let ip_res: Result<(i64, i64, i64), _> = ip_limit_script
            .key(&ip_key)
            .arg(10) // Max 10 attempts
            .arg(60) // 60-second window
            .invoke_async(&mut redis)
            .await;

        if let Ok((allowed, ttl, _)) = ip_res {
            if allowed == 0 {
                crate::metrics::METRICS.counter_inc_labeled("auth_login_failures_total", "reason", "rate_limited");
                return Err(ApiError::RateLimited {
                    retry_after: if ttl > 0 { ttl as u64 } else { 60 },
                });
            }
        }

        // Check email failure lock (5 failed attempts / 60 seconds)
        let email_count: Result<Option<i64>, _> = redis.get(&email_key).await;
        if let Ok(Some(count)) = email_count {
            if count >= 5 {
                crate::metrics::METRICS.counter_inc_labeled("auth_login_failures_total", "reason", "rate_limited");
                let ttl: i64 = redis.ttl(&email_key).await.unwrap_or(60);
                return Err(ApiError::RateLimited {
                    retry_after: if ttl > 0 { ttl as u64 } else { 60 },
                });
            }
        }
    }

    // 3. Lookup user by normalized email
    let user_opt: Option<User> = sqlx::query_as(
        r#"
        SELECT id, email, email_normalized, name, role, status,
               email_verified_at, auth_version, created_at, updated_at, metadata
        FROM users
        WHERE email_normalized = $1
        "#,
    )
    .bind(&email_norm)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    let user = match user_opt {
        Some(u) => u,
        None => {
            // User not found: execute constant-time dummy verify to neutralize timing side channels
            let gate = state.argon2_gate();
            let _ = gate.run(|| crypto::password::dummy_verify()).await;

            // Record failed attempt in Redis for email spray clamping
            if let Some(mut redis) = state.redis.clone() {
                let fail_script = Script::new(
                    r#"
                    local current = redis.call('INCR', KEYS[1])
                    if tonumber(current) == 1 then
                        redis.call('EXPIRE', KEYS[1], ARGV[2])
                    end
                    return current
                    "#,
                );
                let _: Result<i64, _> = fail_script.key(&email_key).arg(5).arg(60).invoke_async(&mut redis).await;
            }

            // Write failure audit event directly via db_pool [REQUIRED-1]
            record_login_failure(
                &state,
                None,
                "auth.login.failed_credentials",
                client_ip,
                &user_agent,
                serde_json::json!({ "attempted_email_domain": email_domain }),
            )
            .await;

            return Err(ApiError::UnauthorizedWithCode {
                code: "invalid_credentials".to_string(),
                message: "Invalid email or password.".to_string(),
            });
        }
    };

    // 4. Lookup credentials
    let cred_row: Option<(String,)> = sqlx::query_as(
        "SELECT secret_hash FROM credentials WHERE user_id = $1 AND kind = 'password' AND provider = 'local'",
    )
    .bind(user.id.into_inner())
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    let secret_hash = match cred_row {
        Some((hash,)) => hash,
        None => {
            let gate = state.argon2_gate();
            let _ = gate.run(|| crypto::password::dummy_verify()).await;

            record_login_failure(
                &state,
                Some(user.id.into_inner()),
                "auth.login.failed_credentials",
                client_ip,
                &user_agent,
                serde_json::json!({ "attempted_email_domain": email_domain }),
            )
            .await;

            return Err(ApiError::UnauthorizedWithCode {
                code: "invalid_credentials".to_string(),
                message: "Invalid email or password.".to_string(),
            });
        }
    };

    // 5. Verify password through Argon2Gate with zeroize-on-drop [NICE-4]
    let mut password_clone = payload.password.clone();
    let gate = state.argon2_gate();
    let is_valid = match gate
        .run(move || {
            use zeroize::Zeroize;
            let res = crypto::password::verify_password(&password_clone, &secret_hash);
            password_clone.zeroize();
            res
        })
        .await
    {
        Ok(Ok(valid)) => valid,
        Ok(Err(_)) => false,
        Err(crate::middleware::argon2_gate::GateError::Timeout) => {
            return Err(ApiError::ServiceUnavailable(
                "Server is under heavy load, please retry shortly".to_string(),
            ));
        }
        Err(_) => {
            return Err(ApiError::Internal("Password verification failed".to_string()));
        }
    };

    if !is_valid {
        if let Some(mut redis) = state.redis.clone() {
            let fail_script = Script::new(
                r#"
                local current = redis.call('INCR', KEYS[1])
                if tonumber(current) == 1 then
                    redis.call('EXPIRE', KEYS[1], ARGV[2])
                end
                return current
                "#,
            );
            let _: Result<i64, _> = fail_script.key(&email_key).arg(5).arg(60).invoke_async(&mut redis).await;
        }

        record_login_failure(
            &state,
            Some(user.id.into_inner()),
            "auth.login.failed_credentials",
            client_ip,
            &user_agent,
            serde_json::json!({ "attempted_email_domain": email_domain }),
        )
        .await;

        return Err(ApiError::UnauthorizedWithCode {
            code: "invalid_credentials".to_string(),
            message: "Invalid email or password.".to_string(),
        });
    }

    // 6. Check user status
    if user.status != UserStatus::Active {
        record_login_failure(
            &state,
            Some(user.id.into_inner()),
            "auth.login.failed_suspended",
            client_ip,
            &user_agent,
            serde_json::json!({ "status": user.status.as_str() }),
        )
        .await;

        return Err(ApiError::ForbiddenWithCode {
            code: "account_suspended".to_string(),
            message: "Account is suspended or pending verification.".to_string(),
        });
    }

    // 7. Check MFA enrollment (Phase 6B)
    let mfa_cred: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT id FROM mfa_credentials
        WHERE user_id = $1 AND confirmed_at IS NOT NULL
        "#,
    )
    .bind(user.id.into_inner())
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to query mfa credentials: {:?}", e)))?;

    if mfa_cred.is_some() {
        record_audit_event(
            &state,
            Some(user.id.into_inner()),
            "auth.mfa.challenge_issued",
            client_ip,
            &user_agent,
            serde_json::json!({ "user_id": user.id.into_inner().to_string() }),
        )
        .await;

        let jti = Uuid::now_v7();
        if let Some(mut redis) = state.redis.clone() {
            let challenge_key = format!("mfa:challenge:{}", jti);
            let _: Result<(), _> = redis.set_ex(&challenge_key, user.id.into_inner().to_string(), 300).await;
        }

        let now_ts = chrono::Utc::now().timestamp();
        let claims = crypto::jwt::Claims {
            sub: user.id.into_inner().to_string(),
            iss: crypto::jwt::DEFAULT_ISSUER.to_string(),
            aud: crypto::jwt::DEFAULT_AUDIENCE.to_string(),
            exp: now_ts + 300,
            iat: now_ts,
            nbf: now_ts,
            jti,
            sid: Uuid::nil(),
            auth_version: user.auth_version,
            role: user.role.to_string(),
            scope: "mfa_pending".to_string(),
        };

        let mfa_token = state
            .jwt_keys
            .sign_access_token(&claims)
            .map_err(|e| ApiError::Internal(format!("Failed to sign mfa token: {:?}", e)))?;

        let login_response = LoginResponse {
            mfa_required: true,
            mfa_enrollment_required: None,
            enrollment_token: None,
            expires_in: None,
            mfa_token: Some(mfa_token),
            access_token: None,
            user: None,
        };

        return Ok(Json(login_response).into_response());
    } else if user.role == UserRole::Moderator || user.role == UserRole::Admin {
        let now_ts = chrono::Utc::now().timestamp();
        let claims = crypto::jwt::Claims {
            sub: user.id.into_inner().to_string(),
            iss: crypto::jwt::DEFAULT_ISSUER.to_string(),
            aud: crypto::jwt::DEFAULT_AUDIENCE.to_string(),
            exp: now_ts + 900,
            iat: now_ts,
            nbf: now_ts,
            jti: Uuid::now_v7(),
            sid: Uuid::nil(),
            auth_version: user.auth_version,
            role: user.role.to_string(),
            scope: "mfa_enrollment".to_string(),
        };

        let enrollment_token = state
            .jwt_keys
            .sign_access_token(&claims)
            .map_err(|e| ApiError::Internal(format!("Failed to sign enrollment token: {:?}", e)))?;

        record_audit_event(
            &state,
            Some(user.id.into_inner()),
            "auth.login.failed_mfa_required",
            client_ip,
            &user_agent,
            serde_json::json!({ "role": user.role.as_str() }),
        )
        .await;

        let login_response = LoginResponse {
            mfa_required: false,
            mfa_enrollment_required: Some(true),
            enrollment_token: Some(enrollment_token),
            expires_in: Some(900),
            mfa_token: None,
            access_token: None,
            user: None,
        };

        return Ok((StatusCode::OK, Json(login_response)).into_response());
    }

    // 8. Successful Customer login without MFA: reset email rate limit counter
    if let Some(mut redis) = state.redis.clone() {
        let _: Result<(), _> = redis.del(&email_key).await;
    }

    // 9. Issue session, refresh cookie, and access token
    issue_login_session(
        &state,
        &user,
        client_ip,
        user_agent,
        "auth.login.success",
        serde_json::json!({ "role": user.role.as_str() }),
    )
    .await
}

/// Issues session, refresh token, sets __Secure-has_refresh cookie, records audit event,
/// and returns LoginResponse with access_token. Shared by /auth/login, /auth/login/totp, and /auth/login/recovery.
pub async fn issue_login_session(
    state: &AppState,
    user: &User,
    client_ip: std::net::IpAddr,
    user_agent: String,
    event: &str,
    metadata: serde_json::Value,
) -> Result<Response, ApiError> {
    let session_id = SessionId::now_v7();
    let (raw_refresh, token_hash) = crypto::token::generate_refresh_token();
    let refresh_id = RefreshTokenId::now_v7();
    let family_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let expires_at = now + chrono::Duration::days(30);

    let mut tx = state
        .db_pool
        .begin()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to start transaction: {:?}", e)))?;

    // Insert session
    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, device_fingerprint, user_agent, ip_address, created_at, last_seen_at, expires_at, revoked_at)
        VALUES ($1, $2, NULL, $3, $4, $5, $5, $6, NULL)
        "#,
    )
    .bind(session_id.into_inner())
    .bind(user.id.into_inner())
    .bind(&user_agent)
    .bind(client_ip)
    .bind(now)
    .bind(expires_at)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to create session: {:?}", e)))?;

    // Insert refresh token
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (id, session_id, family_id, token_hash, issued_at, expires_at, revoked_at, replaced_by_id)
        VALUES ($1, $2, $3, $4, $5, $6, NULL, NULL)
        "#,
    )
    .bind(refresh_id.into_inner())
    .bind(session_id.into_inner())
    .bind(family_id)
    .bind(&token_hash)
    .bind(now)
    .bind(expires_at)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to create refresh token: {:?}", e)))?;

    // Insert audit log event
    crate::metrics::METRICS.counter_inc_labeled("auth_audit_events_total", "event", event);
    let audit_id = AuditLogId::now_v7();
    sqlx::query(
        r#"
        INSERT INTO audit_log (id, user_id, event, ip_address, user_agent, metadata, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(audit_id.into_inner())
    .bind(user.id.into_inner())
    .bind(event)
    .bind(client_ip)
    .bind(&user_agent)
    .bind(metadata)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to write audit event: {:?}", e)))?;

    tx.commit()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to commit login transaction: {:?}", e)))?;

    // 10. Best-effort cache population in Redis for zero-DB warm cache lookups
    if let Some(mut redis) = state.redis.clone() {
        let auth_version_key = format!("auth_version:{}", user.id.into_inner());
        let user_status_key = format!("user_status:{}", user.id.into_inner());
        let _: Result<(), _> = redis.set_ex(&auth_version_key, user.auth_version, 300).await;
        let _: Result<(), _> = redis.set_ex(&user_status_key, user.status.as_str(), 300).await;
    }

    // 11. Sign RS256 access token with JwtKeys
    let now_ts = now.timestamp();
    let claims = crypto::jwt::Claims {
        sub: user.id.into_inner().to_string(),
        iss: crypto::jwt::DEFAULT_ISSUER.to_string(),
        aud: crypto::jwt::DEFAULT_AUDIENCE.to_string(),
        exp: now_ts + 900, // 15-minute validity
        iat: now_ts,
        nbf: now_ts,
        jti: Uuid::now_v7(),
        sid: session_id.into_inner(),
        auth_version: user.auth_version,
        role: user.role.to_string(),
        scope: "access".to_string(),
    };

    let access_token = state
        .jwt_keys
        .sign_access_token(&claims)
        .map_err(|e| ApiError::Internal(format!("Failed to sign access token: {:?}", e)))?;

    // 12. Build response with __Secure-has_refresh cookie
    let user_dto = UserDto {
        id: user.id.into_inner(),
        email: user.email.clone(),
        name: user.name.clone(),
        role: user.role.to_string(),
        status: user.status.to_string(),
        email_verified: user.email_verified_at.is_some(),
    };

    let login_response = LoginResponse {
        mfa_required: false,
        mfa_enrollment_required: None,
        enrollment_token: None,
        expires_in: None,
        mfa_token: None,
        access_token: Some(access_token),
        user: Some(user_dto),
    };

    let cookie_val = format!(
        "__Secure-has_refresh={}; Path=/auth; Max-Age={}; HttpOnly; Secure; SameSite=Lax",
        raw_refresh,
        30 * 24 * 3600
    );

    let mut response = (StatusCode::OK, Json(login_response)).into_response();
    if let Ok(header_val) = axum::http::HeaderValue::from_str(&cookie_val) {
        response.headers_mut().insert(axum::http::header::SET_COOKIE, header_val);
    }

    Ok(response)
}

/// Handler 2: GET /auth/me
/// Returns UserDto for the currently authenticated user.
pub async fn me(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<UserDto>, ApiError> {
    let user_opt: Option<User> = sqlx::query_as(
        r#"
        SELECT id, email, email_normalized, name, role, status,
               email_verified_at, auth_version, created_at, updated_at, metadata
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(auth_user.user_id.into_inner())
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    let user = user_opt.ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    Ok(Json(UserDto {
        id: user.id.into_inner(),
        email: user.email,
        name: user.name,
        role: user.role.to_string(),
        status: user.status.to_string(),
        email_verified: user.email_verified_at.is_some(),
    }))
}

fn spawn_email_dispatch(email_adapter: std::sync::Arc<dyn EmailPort>, email_msg: EmailMessage) {
    tokio::spawn(async move {
        let start = std::time::Instant::now();
        match email_adapter.send(email_msg).await {
            Ok(_) => {
                let elapsed = start.elapsed().as_secs_f64();
                crate::metrics::METRICS.histogram_observe("auth_email_dispatch_latency_seconds", elapsed);
            }
            Err(e) => {
                tracing::error!("Async email dispatch failed: {:?}", e);
                crate::metrics::METRICS.counter_inc("auth_email_dispatch_failure_total");
            }
        }
    });
}

/// Handler 3: POST /auth/register
/// Enforces uniform 202 response, Argon2Gate hashing, and async OTP dispatch.
pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RegisterRequest>,
) -> Result<Response, ApiError> {
    payload.validate().map_err(|e| ApiError::BadRequestWithCode {
        code: "validation_error".to_string(),
        message: e.to_string(),
    })?;

    let ip = extract_client_ip(&headers);
    let masked_ip = mask_ip(&ip);
    let user_agent = extract_user_agent(&headers);
    let rl_key = format!("rl:register:ip:{}", masked_ip);

    if let Some(mut redis) = state.redis.clone() {
        let attempts: i64 = redis.incr(&rl_key, 1).await.unwrap_or(1);
        if attempts == 1 {
            let _: Result<(), _> = redis.expire(&rl_key, 3600).await;
        }
        if attempts > 3 {
            return Err(ApiError::RateLimited { retry_after: 3600 });
        }
    }

    let normalized_email = payload.email.trim().to_lowercase();
    let existing_user: Option<User> = sqlx::query_as(
        r#"
        SELECT id, email, email_normalized, name, role, status,
               email_verified_at, auth_version, created_at, updated_at, metadata
        FROM users
        WHERE email_normalized = $1
        "#,
    )
    .bind(&normalized_email)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    if let Some(user) = existing_user {
        if user.status != UserStatus::Pending {
            // Branch 1: Existing non-pending user (Active, Suspended, Deleted)
            // 1. Argon2 gate call first (~50ms) to ensure timing uniformity
            let _ = state.argon2_gate().run(|| crypto::password::dummy_verify()).await;

            // 2. DB operations (audit event log)
            record_audit_event(
                &state,
                Some(user.id.into_inner()),
                "auth.register.requested",
                ip,
                &user_agent,
                serde_json::json!({
                    "email_domain": extract_email_domain(&payload.email),
                    "ip": masked_ip,
                    "user_agent": user_agent,
                }),
            )
            .await;

            // 3. No email dispatch
            // 4. Return 202
            return Ok((
                StatusCode::ACCEPTED,
                Json(RegisterResponse {
                    status: "pending_verification".to_string(),
                    message: "If eligible, a verification code has been dispatched.".to_string(),
                }),
            ).into_response());
        }

        // Branch 2: Pending user -> re-generate OTP code, overwrite in Redis, dispatch email
        // 1. Argon2 gate call first (~50ms) BEFORE DB/Redis work per [REQUIRED-1]
        let _ = state.argon2_gate().run(|| crypto::password::dummy_verify()).await;

        // 2. DB / Redis operations
        let user_id = user.id.into_inner();
        let code = crypto::otp::generate_otp_code();
        let hash = crypto::otp::hash_otp(&state.email_pepper, user_id, &code);

        if let Some(mut redis) = state.redis.clone() {
            let otp_key = format!("otp:email_verify:{}", user_id);
            let verify_rl_key = format!("rl:otp_verify:{}", user_id);
            let send_count_key = format!("rl:otp_send_count:{}", user_id);
            let send_time_key = format!("rl:otp_send_last:{}", user_id);

            let _: Result<(), _> = redis.set_ex(&otp_key, &hash, 600).await;
            let _: Result<(), _> = redis.del(&verify_rl_key).await;
            let _: Result<(), _> = redis.set_ex(&send_count_key, 1, 600).await;
            let _: Result<(), _> = redis.set_ex(&send_time_key, chrono::Utc::now().timestamp(), 600).await;
        }

        record_audit_event(
            &state,
            Some(user_id),
            "auth.register.requested",
            ip,
            &user_agent,
            serde_json::json!({
                "email_domain": extract_email_domain(&payload.email),
                "ip": masked_ip,
                "user_agent": user_agent,
            }),
        )
        .await;

        // 3. spawn_dispatch (non-blocking fire-and-forget)
        let email_msg = EmailMessage {
            to: payload.email.clone(),
            subject: "Verify your email".to_string(),
            template_id: "email_verify_otp".to_string(),
            variables: serde_json::json!({ "code": code }),
        };
        spawn_email_dispatch(state.email_adapter.clone(), email_msg);

        // 4. Return 202
        return Ok((
            StatusCode::ACCEPTED,
            Json(RegisterResponse {
                status: "pending_verification".to_string(),
                message: "If eligible, a verification code has been dispatched.".to_string(),
            }),
        ).into_response());
    }

    // Branch 3: New user
    // 1. Argon2 gate call first (~50ms)
    let pw = payload.password.clone();
    let secret_hash = state
        .argon2_gate()
        .run(move || crypto::password::hash_password(&pw))
        .await
        .map_err(|e| ApiError::ServiceUnavailable(e.to_string()))?
        .map_err(|e| ApiError::Internal(format!("Password hashing failed: {:?}", e)))?;

    // 2. DB / Redis operations
    let new_user_id = Uuid::now_v7();
    let name = payload.name.clone().unwrap_or_else(|| {
        payload.email.split('@').next().unwrap_or("Customer").to_string()
    });

    let mut tx = state
        .db_pool
        .begin()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to start transaction: {:?}", e)))?;

    sqlx::query(
        r#"
        INSERT INTO users (id, email, email_normalized, name, role, status, metadata, created_at, updated_at)
        VALUES ($1, $2, $3, $4, 'CUSTOMER', 'PENDING', '{}'::jsonb, now(), now())
        "#,
    )
    .bind(new_user_id)
    .bind(&payload.email)
    .bind(&normalized_email)
    .bind(&name)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to insert user: {:?}", e)))?;

    let cred_id = Uuid::now_v7();
    sqlx::query(
        r#"
        INSERT INTO credentials (id, user_id, kind, provider, provider_subject, secret_hash, created_at, updated_at)
        VALUES ($1, $2, 'password', 'local', NULL, $3, now(), now())
        "#,
    )
    .bind(cred_id)
    .bind(new_user_id)
    .bind(&secret_hash)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to insert credentials: {:?}", e)))?;

    tx.commit()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to commit registration transaction: {:?}", e)))?;

    let code = crypto::otp::generate_otp_code();
    let hash = crypto::otp::hash_otp(&state.email_pepper, new_user_id, &code);

    if let Some(mut redis) = state.redis.clone() {
        let otp_key = format!("otp:email_verify:{}", new_user_id);
        let verify_rl_key = format!("rl:otp_verify:{}", new_user_id);
        let send_count_key = format!("rl:otp_send_count:{}", new_user_id);
        let send_time_key = format!("rl:otp_send_last:{}", new_user_id);

        let _: Result<(), _> = redis.set_ex(&otp_key, &hash, 600).await;
        let _: Result<(), _> = redis.del(&verify_rl_key).await;
        let _: Result<(), _> = redis.set_ex(&send_count_key, 1, 600).await;
        let _: Result<(), _> = redis.set_ex(&send_time_key, chrono::Utc::now().timestamp(), 600).await;
    }

    record_audit_event(
        &state,
        Some(new_user_id),
        "auth.register.requested",
        ip,
        &user_agent,
        serde_json::json!({
            "email_domain": extract_email_domain(&payload.email),
            "ip": masked_ip,
            "user_agent": user_agent,
        }),
    )
    .await;

    // 3. spawn_dispatch (non-blocking fire-and-forget)
    let email_msg = EmailMessage {
        to: payload.email.clone(),
        subject: "Verify your email".to_string(),
        template_id: "email_verify_otp".to_string(),
        variables: serde_json::json!({ "code": code }),
    };
    spawn_email_dispatch(state.email_adapter.clone(), email_msg);

    // 4. Return 202
    Ok((
        StatusCode::ACCEPTED,
        Json(RegisterResponse {
            status: "pending_verification".to_string(),
            message: "If eligible, a verification code has been dispatched.".to_string(),
        }),
    ).into_response())
}

/// Handler 4: POST /auth/register/verify-otp
pub async fn register_verify_otp(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<VerifyOtpRequest>,
) -> Result<Json<VerifyOtpResponse>, ApiError> {
    payload.validate().map_err(|e| ApiError::BadRequestWithCode {
        code: "validation_error".to_string(),
        message: e.to_string(),
    })?;

    let ip = extract_client_ip(&headers);
    let masked_ip = mask_ip(&ip);
    let user_agent = extract_user_agent(&headers);

    let normalized_email = payload.email.trim().to_lowercase();
    let user_opt: Option<User> = sqlx::query_as(
        r#"
        SELECT id, email, email_normalized, name, role, status,
               email_verified_at, auth_version, created_at, updated_at, metadata
        FROM users
        WHERE email_normalized = $1
        "#,
    )
    .bind(&normalized_email)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    let user = match user_opt {
        Some(u) => u,
        None => {
            let _ = state.argon2_gate().run(|| crypto::password::dummy_verify()).await;
            return Err(ApiError::BadRequestWithCode {
                code: "invalid_otp".to_string(),
                message: "Invalid OTP code".to_string(),
            });
        }
    };

    let user_id = user.id.into_inner();
    let otp_key = format!("otp:email_verify:{}", user_id);
    let verify_rl_key = format!("rl:otp_verify:{}", user_id);

    if let Some(mut redis) = state.redis.clone() {
        let attempts: i64 = redis.get(&verify_rl_key).await.unwrap_or(None).unwrap_or(0);
        if attempts >= 5 {
            let _: Result<(), _> = redis.del(&otp_key).await;
            return Err(ApiError::RateLimited {
                retry_after: 600,
            });
        }
    }

    let expected_hash: Option<String> = if let Some(mut redis) = state.redis.clone() {
        redis.get(&otp_key).await.unwrap_or(None)
    } else {
        None
    };

    let expected_hash = match expected_hash {
        Some(h) => h,
        None => {
            return Err(ApiError::BadRequestWithCode {
                code: "otp_expired".to_string(),
                message: "OTP code has expired".to_string(),
            });
        }
    };

    let is_valid = crypto::otp::verify_otp_hash(&state.email_pepper, user_id, &payload.code, &expected_hash);

    if !is_valid {
        if let Some(mut redis) = state.redis.clone() {
            let attempts: i64 = redis.incr(&verify_rl_key, 1).await.unwrap_or(1);
            if attempts == 1 {
                let _: Result<(), _> = redis.expire(&verify_rl_key, 600).await;
            }
            if attempts >= 5 {
                let _: Result<(), _> = redis.del(&otp_key).await;
                return Err(ApiError::BadRequestWithCode {
                    code: "otp_expired".to_string(),
                    message: "OTP code has expired".to_string(),
                });
            }
        }
        return Err(ApiError::BadRequestWithCode {
            code: "invalid_otp".to_string(),
            message: "Invalid OTP code".to_string(),
        });
    }

    // Match -> Activate user
    sqlx::query(
        r#"
        UPDATE users
        SET status = 'ACTIVE', email_verified_at = now(), updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to activate user: {:?}", e)))?;

    if let Some(mut redis) = state.redis.clone() {
        let _: Result<(), _> = redis.del(&otp_key).await;
        let _: Result<(), _> = redis.del(&verify_rl_key).await;
    }

    record_audit_event(
        &state,
        Some(user_id),
        "auth.register.verified",
        ip,
        &user_agent,
        serde_json::json!({
            "user_id": user_id,
            "ip": masked_ip,
            "user_agent": user_agent,
        }),
    )
    .await;

    Ok(Json(VerifyOtpResponse {
        message: "Email verified successfully.".to_string(),
        verified: true,
    }))
}

/// Handler 5: POST /auth/register/resend-otp
pub async fn register_resend_otp(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ResendOtpRequest>,
) -> Result<Json<ResendOtpResponse>, ApiError> {
    payload.validate().map_err(|e| ApiError::BadRequestWithCode {
        code: "validation_error".to_string(),
        message: e.to_string(),
    })?;

    let ip = extract_client_ip(&headers);
    let masked_ip = mask_ip(&ip);
    let user_agent = extract_user_agent(&headers);

    let normalized_email = payload.email.trim().to_lowercase();
    let user_opt: Option<User> = sqlx::query_as(
        r#"
        SELECT id, email, email_normalized, name, role, status,
               email_verified_at, auth_version, created_at, updated_at, metadata
        FROM users
        WHERE email_normalized = $1
        "#,
    )
    .bind(&normalized_email)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    let user = match user_opt {
        Some(u) => u,
        None => {
            return Ok(Json(ResendOtpResponse {
                status: "dispatched".to_string(),
                message: "If eligible, a new verification code has been dispatched.".to_string(),
            }));
        }
    };

    if user.status != UserStatus::Pending {
        return Ok(Json(ResendOtpResponse {
            status: "dispatched".to_string(),
            message: "If eligible, a new verification code has been dispatched.".to_string(),
        }));
    }

    let user_id = user.id.into_inner();
    let count_key = format!("rl:otp_send_count:{}", user_id);
    let time_key = format!("rl:otp_send_last:{}", user_id);
    let otp_key = format!("otp:email_verify:{}", user_id);
    let verify_rl_key = format!("rl:otp_verify:{}", user_id);

    let now = chrono::Utc::now().timestamp();
    if let Some(mut redis) = state.redis.clone() {
        let count: i64 = redis.get(&count_key).await.unwrap_or(None).unwrap_or(0);
        let last_time: Option<i64> = redis.get(&time_key).await.unwrap_or(None);

        if count >= 1 {
            if let Some(lt) = last_time {
                let elapsed = now - lt;
                let required_delay = if count == 1 { 60 } else if count == 2 { 180 } else { 600 };
                if elapsed < required_delay {
                    let retry_after = (required_delay - elapsed) as u64;
                    return Err(ApiError::RateLimited {
                        retry_after: std::cmp::max(retry_after, 1),
                    });
                }
            }
        }
        if count >= 4 {
            return Err(ApiError::RateLimited { retry_after: 600 });
        }

        let _: Result<(), _> = redis.set_ex(&count_key, count + 1, 600).await;
        let _: Result<(), _> = redis.set_ex(&time_key, now, 600).await;
    }

    let code = crypto::otp::generate_otp_code();
    let hash = crypto::otp::hash_otp(&state.email_pepper, user_id, &code);

    if let Some(mut redis) = state.redis.clone() {
        let _: Result<(), _> = redis.set_ex(&otp_key, &hash, 600).await;
        let _: Result<(), _> = redis.del(&verify_rl_key).await;
    }

    let email_msg = EmailMessage {
        to: payload.email.clone(),
        subject: "Verify your email".to_string(),
        template_id: "email_verify_otp".to_string(),
        variables: serde_json::json!({ "code": code }),
    };

    if let Err(e) = state.email_adapter.send(email_msg).await {
        tracing::error!("Email dispatch failed during resend: {:?}", e);
        return Err(ApiError::ServiceUnavailable(
            "Email delivery service is currently unavailable. Please try again later.".to_string(),
        ));
    }

    record_audit_event(
        &state,
        Some(user_id),
        "auth.register.otp_resent",
        ip,
        &user_agent,
        serde_json::json!({
            "user_id": user_id,
            "ip": masked_ip,
            "user_agent": user_agent,
        }),
    )
    .await;

    Ok(Json(ResendOtpResponse {
        status: "dispatched".to_string(),
        message: "A new verification code has been dispatched.".to_string(),
    }))
}

/// Handler 6: POST /auth/password/forgot
pub async fn password_forgot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Response, ApiError> {
    payload.validate().map_err(|e| ApiError::BadRequestWithCode {
        code: "validation_error".to_string(),
        message: e.to_string(),
    })?;

    let ip = extract_client_ip(&headers);
    let masked_ip = mask_ip(&ip);
    let user_agent = extract_user_agent(&headers);
    let normalized_email = payload.email.trim().to_lowercase();

    let ip_rl_key = format!("rl:forgot:ip:{}", masked_ip);
    let email_rl_key = format!("rl:forgot:email:{}", normalized_email);

    if let Some(mut redis) = state.redis.clone() {
        let ip_attempts: i64 = redis.incr(&ip_rl_key, 1).await.unwrap_or(1);
        if ip_attempts == 1 {
            let _: Result<(), _> = redis.expire(&ip_rl_key, 3600).await;
        }
        if ip_attempts > 3 {
            return Err(ApiError::RateLimited { retry_after: 3600 });
        }

        let email_attempts: i64 = redis.incr(&email_rl_key, 1).await.unwrap_or(1);
        if email_attempts == 1 {
            let _: Result<(), _> = redis.expire(&email_rl_key, 300).await;
        }
        if email_attempts > 1 {
            return Err(ApiError::RateLimited { retry_after: 300 });
        }
    }

    let user_opt: Option<User> = sqlx::query_as(
        r#"
        SELECT id, email, email_normalized, name, role, status,
               email_verified_at, auth_version, created_at, updated_at, metadata
        FROM users
        WHERE email_normalized = $1
        "#,
    )
    .bind(&normalized_email)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    if let Some(user) = user_opt {
        if user.status == UserStatus::Active {
            let user_id = user.id.into_inner();
            let code = crypto::otp::generate_otp_code();
            let hash = crypto::otp::hash_otp(&state.email_pepper, user_id, &code);

            if let Some(mut redis) = state.redis.clone() {
                let reset_key = format!("otp:password_reset:{}", user_id);
                let verify_rl_key = format!("rl:otp_verify:{}", user_id);
                let _: Result<(), _> = redis.set_ex(&reset_key, &hash, 600).await;
                let _: Result<(), _> = redis.del(&verify_rl_key).await;
            }

            let email_msg = EmailMessage {
                to: payload.email.clone(),
                subject: "Reset your password".to_string(),
                template_id: "password_reset_otp".to_string(),
                variables: serde_json::json!({ "code": code }),
            };

            if let Err(e) = state.email_adapter.send(email_msg).await {
                tracing::error!("Email dispatch failed during password forgot: {:?}", e);
                return Err(ApiError::ServiceUnavailable(
                    "Email delivery service is currently unavailable. Please try again later.".to_string(),
                ));
            }

            record_audit_event(
                &state,
                Some(user_id),
                "auth.password.reset_requested",
                ip,
                &user_agent,
                serde_json::json!({
                    "email_domain": extract_email_domain(&payload.email),
                    "ip": masked_ip,
                    "user_agent": user_agent,
                }),
            )
            .await;
        } else {
            let _ = state.argon2_gate().run(|| crypto::password::dummy_verify()).await;
        }
    } else {
        let _ = state.argon2_gate().run(|| crypto::password::dummy_verify()).await;
    }

    Ok((
        StatusCode::ACCEPTED,
        Json(ForgotPasswordResponse {
            status: "accepted".to_string(),
            message: "If this email is eligible, password reset instructions have been dispatched.".to_string(),
        }),
    ).into_response())
}

/// Handler 7: POST /auth/password/reset
pub async fn password_reset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<Json<ResetPasswordResponse>, ApiError> {
    payload.validate().map_err(|e| ApiError::BadRequestWithCode {
        code: "validation_error".to_string(),
        message: e.to_string(),
    })?;

    let ip = extract_client_ip(&headers);
    let masked_ip = mask_ip(&ip);
    let user_agent = extract_user_agent(&headers);

    let normalized_email = payload.email.trim().to_lowercase();
    let user_opt: Option<User> = sqlx::query_as(
        r#"
        SELECT id, email, email_normalized, name, role, status,
               email_verified_at, auth_version, created_at, updated_at, metadata
        FROM users
        WHERE email_normalized = $1
        "#,
    )
    .bind(&normalized_email)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    let user = match user_opt {
        Some(u) => u,
        None => {
            let _ = state.argon2_gate().run(|| crypto::password::dummy_verify()).await;
            return Err(ApiError::BadRequestWithCode {
                code: "invalid_otp".to_string(),
                message: "Invalid OTP code".to_string(),
            });
        }
    };

    let user_id = user.id.into_inner();
    let reset_key = format!("otp:password_reset:{}", user_id);
    let verify_rl_key = format!("rl:otp_verify:{}", user_id);

    if let Some(mut redis) = state.redis.clone() {
        let attempts: i64 = redis.get(&verify_rl_key).await.unwrap_or(None).unwrap_or(0);
        if attempts >= 5 {
            let _: Result<(), _> = redis.del(&reset_key).await;
            return Err(ApiError::RateLimited {
                retry_after: 600,
            });
        }
    }

    let expected_hash: Option<String> = if let Some(mut redis) = state.redis.clone() {
        redis.get(&reset_key).await.unwrap_or(None)
    } else {
        None
    };

    let expected_hash = match expected_hash {
        Some(h) => h,
        None => {
            return Err(ApiError::BadRequestWithCode {
                code: "invalid_otp".to_string(),
                message: "Invalid OTP code".to_string(),
            });
        }
    };

    let is_valid = crypto::otp::verify_otp_hash(&state.email_pepper, user_id, &payload.code, &expected_hash);

    if !is_valid {
        if let Some(mut redis) = state.redis.clone() {
            let attempts: i64 = redis.incr(&verify_rl_key, 1).await.unwrap_or(1);
            if attempts == 1 {
                let _: Result<(), _> = redis.expire(&verify_rl_key, 600).await;
            }
            if attempts >= 5 {
                let _: Result<(), _> = redis.del(&reset_key).await;
                return Err(ApiError::BadRequestWithCode {
                    code: "otp_expired".to_string(),
                    message: "Too many failed attempts. Reset code invalidated.".to_string(),
                });
            }
        }
        return Err(ApiError::BadRequestWithCode {
            code: "invalid_otp".to_string(),
            message: "Invalid OTP code".to_string(),
        });
    }

    let new_pw = payload.new_password.clone();
    let new_secret_hash = state
        .argon2_gate()
        .run(move || crypto::password::hash_password(&new_pw))
        .await
        .map_err(|e| ApiError::ServiceUnavailable(e.to_string()))?
        .map_err(|e| ApiError::Internal(format!("Password hashing failed: {:?}", e)))?;

    // Atomic transaction: update credentials, bump auth_version, revoke all sessions & refresh tokens
    let mut tx = state
        .db_pool
        .begin()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to start transaction: {:?}", e)))?;

    sqlx::query(
        r#"
        UPDATE credentials
        SET secret_hash = $1, updated_at = now()
        WHERE user_id = $2 AND kind = 'password' AND provider = 'local'
        "#,
    )
    .bind(&new_secret_hash)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to update credentials: {:?}", e)))?;

    sqlx::query(
        r#"
        UPDATE users
        SET auth_version = auth_version + 1, updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to bump auth_version: {:?}", e)))?;

    sqlx::query(
        r#"
        UPDATE sessions
        SET revoked_at = now()
        WHERE user_id = $1 AND revoked_at IS NULL
        "#,
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to revoke sessions: {:?}", e)))?;

    sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = now()
        WHERE session_id IN (SELECT id FROM sessions WHERE user_id = $1) AND revoked_at IS NULL
        "#,
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to revoke refresh tokens: {:?}", e)))?;

    tx.commit()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to commit password reset transaction: {:?}", e)))?;

    // Post-commit Redis cleanup
    if let Some(mut redis) = state.redis.clone() {
        let auth_version_key = format!("auth_version:{}", user_id);
        let _: Result<(), _> = redis.del(&auth_version_key).await;
        let _: Result<(), _> = redis.del(&reset_key).await;
        let _: Result<(), _> = redis.del(&verify_rl_key).await;
    }

    record_audit_event(
        &state,
        Some(user_id),
        "auth.password.reset_completed",
        ip,
        &user_agent,
        serde_json::json!({
            "user_id": user_id,
            "ip": masked_ip,
            "user_agent": user_agent,
        }),
    )
    .await;

    Ok(Json(ResetPasswordResponse {
        message: "Password reset successfully.".to_string(),
        success: true,
    }))
}

/// Handler 8: POST /auth/password/change
pub async fn password_change(
    auth_user: AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<ChangePasswordResponse>, ApiError> {
    payload.validate().map_err(|e| ApiError::BadRequestWithCode {
        code: "validation_error".to_string(),
        message: e.to_string(),
    })?;

    let ip = extract_client_ip(&headers);
    let masked_ip = mask_ip(&ip);
    let user_agent = extract_user_agent(&headers);
    let user_id = auth_user.user_id.into_inner();
    let calling_session_id = auth_user.session_id.into_inner();

    let row: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT secret_hash
        FROM credentials
        WHERE user_id = $1 AND kind = 'password' AND provider = 'local'
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error: {:?}", e)))?;

    let (current_secret_hash,) = match row {
        Some(r) => r,
        None => {
            return Err(ApiError::UnauthorizedWithCode {
                code: "invalid_credentials".to_string(),
                message: "Credentials row not found".to_string(),
            });
        }
    };

    let cur_pw = payload.current_password.clone();
    let cur_hash = current_secret_hash.clone();
    let is_valid = state
        .argon2_gate()
        .run(move || crypto::password::verify_password(&cur_pw, &cur_hash))
        .await
        .map_err(|e| ApiError::ServiceUnavailable(e.to_string()))?
        .map_err(|e| ApiError::Internal(format!("Password verification failed: {:?}", e)))?;

    if !is_valid {
        return Err(ApiError::UnauthorizedWithCode {
            code: "invalid_credentials".to_string(),
            message: "Current password is incorrect".to_string(),
        });
    }

    let new_pw = payload.new_password.clone();
    let new_secret_hash = state
        .argon2_gate()
        .run(move || crypto::password::hash_password(&new_pw))
        .await
        .map_err(|e| ApiError::ServiceUnavailable(e.to_string()))?
        .map_err(|e| ApiError::Internal(format!("Password hashing failed: {:?}", e)))?;

    let mut tx = state
        .db_pool
        .begin()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to start transaction: {:?}", e)))?;

    sqlx::query(
        r#"
        UPDATE credentials
        SET secret_hash = $1, updated_at = now()
        WHERE user_id = $2 AND kind = 'password' AND provider = 'local'
        "#,
    )
    .bind(&new_secret_hash)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to update credentials: {:?}", e)))?;

    let bumped_version: (i64,) = sqlx::query_as(
        r#"
        UPDATE users
        SET auth_version = auth_version + 1, updated_at = now()
        WHERE id = $1
        RETURNING auth_version
        "#,
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to bump auth_version: {:?}", e)))?;

    // Revoke all OTHER sessions; keep calling session alive
    sqlx::query(
        r#"
        UPDATE sessions
        SET revoked_at = now()
        WHERE user_id = $1 AND id != $2 AND revoked_at IS NULL
        "#,
    )
    .bind(user_id)
    .bind(calling_session_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to revoke other sessions: {:?}", e)))?;

    // Revoke all OTHER refresh tokens
    sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = now()
        WHERE session_id IN (SELECT id FROM sessions WHERE user_id = $1 AND id != $2) AND revoked_at IS NULL
        "#,
    )
    .bind(user_id)
    .bind(calling_session_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to revoke other refresh tokens: {:?}", e)))?;

    tx.commit()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to commit password change: {:?}", e)))?;

    // Post-commit Redis invalidation
    if let Some(mut redis) = state.redis.clone() {
        let auth_version_key = format!("auth_version:{}", user_id);
        let _: Result<(), _> = redis.del(&auth_version_key).await;
    }

    // Issue successor access token with bumped auth_version for calling session
    let now = chrono::Utc::now().timestamp();
    let claims = crypto::jwt::Claims {
        sub: user_id.to_string(),
        role: auth_user.role.to_string(),
        auth_version: bumped_version.0,
        sid: calling_session_id,
        scope: "access".to_string(),
        exp: now + 900,
        iat: now,
        nbf: now,
        iss: "HAS.LTD".to_string(),
        aud: "HAS.LTD".to_string(),
        jti: Uuid::new_v4(),
    };

    let new_token = state
        .jwt_keys
        .sign_access_token(&claims)
        .map_err(|e| ApiError::Internal(format!("Failed to sign access token: {:?}", e)))?;

    record_audit_event(
        &state,
        Some(user_id),
        "auth.password.changed",
        ip,
        &user_agent,
        serde_json::json!({
            "user_id": user_id,
            "session_id": calling_session_id,
            "ip": masked_ip,
            "user_agent": user_agent,
        }),
    )
    .await;

    Ok(Json(ChangePasswordResponse {
        message: "Password changed successfully.".to_string(),
        success: true,
        access_token: Some(new_token),
    }))
}
