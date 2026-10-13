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
use crate::dtos::{LoginRequest, LoginResponse, UserDto};
use crate::errors::ApiError;
use crate::middleware::auth_guard::AuthUser;
use crate::state::AppState;

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

fn extract_client_ip(headers: &HeaderMap) -> IpAddr {
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

fn extract_user_agent(headers: &HeaderMap) -> String {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
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
    // 1. Validate payload
    payload.validate().map_err(|e| {
        ApiError::BadRequestWithCode {
            code: "bad_request".to_string(),
            message: e.to_string(),
        }
    })?;

    let client_ip = extract_client_ip(&headers);
    let masked_ip = mask_ip(&client_ip);
    let user_agent = extract_user_agent(&headers);
    let email_norm = payload.email.trim().to_ascii_lowercase();

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
                return Err(ApiError::RateLimited {
                    retry_after: if ttl > 0 { ttl as u64 } else { 60 },
                });
            }
        }

        // Check email failure lock (5 failed attempts / 60 seconds)
        let email_count: Result<Option<i64>, _> = redis.get(&email_key).await;
        if let Ok(Some(count)) = email_count {
            if count >= 5 {
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

            return Err(ApiError::UnauthorizedWithCode {
                code: "invalid_credentials".to_string(),
                message: "Invalid email or password.".to_string(),
            });
        }
    };

    // 5. Verify password through Argon2Gate
    let password_clone = payload.password.clone();
    let gate = state.argon2_gate();
    let is_valid = match gate
        .run(move || crypto::password::verify_password(&password_clone, &secret_hash))
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

        return Err(ApiError::UnauthorizedWithCode {
            code: "invalid_credentials".to_string(),
            message: "Invalid email or password.".to_string(),
        });
    }

    // 6. Check user status
    if user.status != UserStatus::Active {
        return Err(ApiError::ForbiddenWithCode {
            code: "account_suspended".to_string(),
            message: "Account is suspended or pending verification.".to_string(),
        });
    }

    // 7. Temporary Phase 4 branch for Moderator and Admin roles
    if user.role == UserRole::Moderator || user.role == UserRole::Admin {
        // TODO: Phase 6 MFA Enrollment & Enforcement (Issue #AUTH-MFA-001)
        return Err(ApiError::NotImplemented {
            code: "mfa_not_yet_available".to_string(),
            message: "Multi-factor authentication is required for staff roles but is not yet available".to_string(),
        });
    }

    // 8. Successful Customer login: reset email rate limit counter
    if let Some(mut redis) = state.redis.clone() {
        let _: Result<(), _> = redis.del(&email_key).await;
    }

    // 9. Single transaction: create session + refresh token + audit log
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
    let audit_id = AuditLogId::now_v7();
    sqlx::query(
        r#"
        INSERT INTO audit_log (id, user_id, event, ip_address, user_agent, metadata, created_at)
        VALUES ($1, $2, 'auth.login.success', $3, $4, $5, $6)
        "#,
    )
    .bind(audit_id.into_inner())
    .bind(user.id.into_inner())
    .bind(client_ip)
    .bind(&user_agent)
    .bind(serde_json::json!({ "role": user.role.as_str() }))
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
        email: user.email,
        name: user.name,
        role: user.role.to_string(),
        status: user.status.to_string(),
        email_verified: user.email_verified_at.is_some(),
    };

    let login_response = LoginResponse {
        mfa_required: false,
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
