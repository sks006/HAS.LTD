// backend/api/src/handlers/session.rs

use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use redis::AsyncCommands;
use uuid::Uuid;

use domain::models::identifiers::{AuditLogId, RefreshTokenId};
use crate::dtos::{LogoutAllResponse, RefreshResponse, SessionDto, SuccessResponse};
use crate::errors::ApiError;
use crate::metrics::METRICS;
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

fn extract_refresh_cookie(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    for part in cookie_header.split(';') {
        let trimmed = part.trim();
        if let Some(rest) = trimmed.strip_prefix("__Secure-has_refresh=") {
            let val = rest.trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

fn hash_refresh_token(raw_b64: &str) -> Option<String> {
    let raw_bytes = URL_SAFE_NO_PAD.decode(raw_b64).ok()?;
    if raw_bytes.len() != 32 {
        return None;
    }
    Some(crypto::token::sha256_hex(&raw_bytes))
}

/// 1. POST /auth/refresh
/// Implements Phase 3 Flow D: Refresh Token Rotation (RTR) with single-flight mutex,
/// 15-second grace period for network retries, and automatic replay detection.
pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    // 1. Extract raw refresh token from __Secure-has_refresh cookie
    let raw_cookie = match extract_refresh_cookie(&headers) {
        Some(c) => c,
        None => {
            METRICS.counter_inc_labeled("auth_token_refresh_total", "result", "missing_cookie");
            return Err(ApiError::UnauthorizedWithCode {
                code: "invalid_token".to_string(),
                message: "Missing refresh cookie".to_string(),
            });
        }
    };

    // 2. Hash token
    let token_hash = match hash_refresh_token(&raw_cookie) {
        Some(h) => h,
        None => {
            METRICS.counter_inc_labeled("auth_token_refresh_total", "result", "invalid_encoding");
            return Err(ApiError::UnauthorizedWithCode {
                code: "token_invalid".to_string(),
                message: "Invalid refresh token encoding".to_string(),
            });
        }
    };

    let client_ip = extract_client_ip(&headers);
    let user_agent = extract_user_agent(&headers);

    // 3. Acquire Redis single-flight lock: lock:refresh:{token_hash} (5s TTL)
    let lock_key = format!("lock:refresh:{}", token_hash);
    let mut redis_lock_held = false;

    if let Some(mut redis) = state.redis.clone() {
        let lock_res: Result<bool, _> = redis.set_options(
            &lock_key,
            "1",
            redis::SetOptions::default()
                .conditional_set(redis::ExistenceCheck::NX)
                .with_expiration(redis::SetExpiry::EX(5)),
        ).await;

        match lock_res {
            Ok(true) => {
                redis_lock_held = true;
            }
            Ok(false) => {
                METRICS.counter_inc_labeled("auth_token_refresh_total", "result", "lock_conflict");
                return Err(ApiError::RefreshInProgress { retry_after: 1 });
            }
            Err(e) => {
                tracing::warn!("Redis error acquiring refresh lock: {}. Proceeding to DB row lock.", e);
            }
        }
    }

    // Defer release helper closure
    let cleanup_lock = |state: AppState, lock_key: String, held: bool| async move {
        if held {
            if let Some(mut redis) = state.redis.clone() {
                let _: Result<(), _> = redis.del(&lock_key).await;
            }
        }
    };

    // 4. Begin single atomic transaction
    let mut tx = match state.db_pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            cleanup_lock(state, lock_key, redis_lock_held).await;
            return Err(ApiError::Internal(format!("Failed to start transaction: {:?}", e)));
        }
    };

    // 5. Select refresh token row FOR UPDATE
    let token_row: Option<(Uuid, Uuid, Uuid, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, Option<Uuid>)> = match sqlx::query_as(
        r#"
        SELECT id, session_id, family_id, token_hash, issued_at, expires_at, revoked_at, replaced_by_id
        FROM refresh_tokens
        WHERE token_hash = $1
        FOR UPDATE
        "#
    )
    .bind(&token_hash)
    .fetch_optional(&mut *tx)
    .await {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.rollback().await;
            cleanup_lock(state, lock_key, redis_lock_held).await;
            return Err(ApiError::Internal(format!("Database query error: {:?}", e)));
        }
    };

    let (old_id, session_id, family_id, _stored_hash, _issued_at, expires_at, revoked_at, replaced_by_id) = match token_row {
        Some(row) => row,
        None => {
            let _ = tx.rollback().await;
            cleanup_lock(state, lock_key, redis_lock_held).await;
            METRICS.counter_inc_labeled("auth_token_refresh_total", "result", "not_found");
            return Err(ApiError::UnauthorizedWithCode {
                code: "token_invalid".to_string(),
                message: "Refresh token is invalid or does not exist".to_string(),
            });
        }
    };

    let now = chrono::Utc::now();

    // 5b. Verify parent session status FOR UPDATE to synchronize with concurrent logout
    let session_row: Option<(Uuid, Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>)> = match sqlx::query_as(
        "SELECT user_id, revoked_at, expires_at FROM sessions WHERE id = $1 FOR UPDATE"
    )
    .bind(session_id)
    .fetch_optional(&mut *tx)
    .await {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.rollback().await;
            cleanup_lock(state, lock_key, redis_lock_held).await;
            return Err(ApiError::Internal(format!("Failed to query session: {:?}", e)));
        }
    };

    let (user_id, sess_revoked_at, sess_expires_at) = match session_row {
        Some(s) => s,
        None => {
            let _ = tx.rollback().await;
            cleanup_lock(state, lock_key, redis_lock_held).await;
            return Err(ApiError::UnauthorizedWithCode {
                code: "token_revoked".to_string(),
                message: "Session not found".to_string(),
            });
        }
    };

    // If parent session was revoked (e.g. via concurrent logout) or expired, fail with token_revoked
    if sess_revoked_at.is_some() || sess_expires_at <= now {
        let _ = tx.rollback().await;
        cleanup_lock(state, lock_key, redis_lock_held).await;
        METRICS.counter_inc_labeled("auth_token_refresh_total", "result", "session_revoked");
        return Err(ApiError::UnauthorizedWithCode {
            code: "token_revoked".to_string(),
            message: "Parent session has been revoked or expired".to_string(),
        });
    }

    // 6. Branch based on revocation status
    if let Some(revoked_timestamp) = revoked_at {
        let elapsed = (now - revoked_timestamp).num_seconds();

        // 6c. Grace window branch: within 15 seconds of replacement [Decision b: Option A]
        if replaced_by_id.is_some() && elapsed <= 15 {
            let grace_key = format!("refresh:grace:{}", token_hash);
            match state.redis.clone() {
                Some(mut redis) => {
                    let cached_val: Result<Option<String>, _> = redis.get(&grace_key).await;
                    match cached_val {
                        // (a) Grace cache hit -> return cached successor (200)
                        Ok(Some(json_str)) => {
                            if let Ok(cached_data) = serde_json::from_str::<serde_json::Value>(&json_str) {
                                let access_token = cached_data["access_token"].as_str().unwrap_or("").to_string();
                                let raw_refresh = cached_data["raw_refresh"].as_str().unwrap_or("").to_string();

                                let _ = tx.rollback().await;
                                cleanup_lock(state, lock_key, redis_lock_held).await;

                                METRICS.counter_inc_labeled("auth_token_refresh_total", "result", "grace_hit");

                                let cookie_val = format!(
                                    "__Secure-has_refresh={}; Path=/auth; Max-Age={}; HttpOnly; Secure; SameSite=Lax",
                                    raw_refresh,
                                    30 * 24 * 3600
                                );
                                let mut res = (StatusCode::OK, Json(RefreshResponse { access_token })).into_response();
                                if let Ok(header_val) = axum::http::HeaderValue::from_str(&cookie_val) {
                                    res.headers_mut().insert(axum::http::header::SET_COOKIE, header_val);
                                }
                                return Ok(res);
                            }
                        }
                        // (b) Cache miss, Redis healthy -> real replay, proceed to family_revoked below
                        Ok(None) => {
                            tracing::warn!("Grace window cache miss for token {} with healthy Redis; treating as replay", token_hash);
                        }
                        // (c) Redis unreachable / error -> 401 token_invalid, do NOT revoke family, do NOT bump auth_version
                        Err(e) => {
                            tracing::warn!("Redis error querying grace cache for {}: {:?}. Failing closed without family revocation.", grace_key, e);
                            let _ = tx.rollback().await;
                            cleanup_lock(state, lock_key, redis_lock_held).await;
                            METRICS.counter_inc_labeled("auth_token_refresh_total", "result", "redis_degraded_grace_miss");
                            return Err(ApiError::UnauthorizedWithCode {
                                code: "token_invalid".to_string(),
                                message: "Refresh token is invalid".to_string(),
                            });
                        }
                    }
                }
                // (c) Redis unreachable (None) -> 401 token_invalid, do NOT revoke family, do NOT bump auth_version
                None => {
                    tracing::warn!("Redis unavailable during grace window check for {}. Failing closed without family revocation.", grace_key);
                    let _ = tx.rollback().await;
                    cleanup_lock(state, lock_key, redis_lock_held).await;
                    METRICS.counter_inc_labeled("auth_token_refresh_total", "result", "redis_degraded_grace_miss");
                    return Err(ApiError::UnauthorizedWithCode {
                        code: "token_invalid".to_string(),
                        message: "Refresh token is invalid".to_string(),
                    });
                }
            }
        }

        // 6d. Replay detection branch:
        // Triggered when revoked > 15s ago, or replaced_by_id IS NULL, or cache miss on healthy Redis
        // Revoke entire family, session, and bump auth_version
        let _ = sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = $1 WHERE family_id = $2 AND revoked_at IS NULL"
        )
        .bind(now)
        .bind(family_id)
        .execute(&mut *tx)
        .await;

        let _ = sqlx::query(
            "UPDATE sessions SET revoked_at = $1 WHERE id = $2"
        )
        .bind(now)
        .bind(session_id)
        .execute(&mut *tx)
        .await;

        let _ = sqlx::query(
            "UPDATE users SET auth_version = auth_version + 1 WHERE id = $1"
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await;

        let audit_id = AuditLogId::now_v7();
        let _ = sqlx::query(
            r#"
            INSERT INTO audit_log (id, user_id, event, ip_address, user_agent, metadata, created_at)
            VALUES ($1, $2, 'auth.session.replay_detected', $3, $4, $5, $6)
            "#
        )
        .bind(audit_id.into_inner())
        .bind(user_id)
        .bind(client_ip)
        .bind(&user_agent)
        .bind(serde_json::json!({
            "session_id": session_id,
            "family_id": family_id,
        }))
        .bind(now)
        .execute(&mut *tx)
        .await;

        let _ = tx.commit().await;

        // Post-commit Redis cache invalidation
        if let Some(mut redis) = state.redis.clone() {
            let _: Result<(), _> = redis.del(format!("auth_version:{}", user_id)).await;
        }

        METRICS.counter_inc("auth_refresh_replay_detected_total");
        METRICS.counter_inc_labeled("auth_audit_events_total", "event", "auth.session.replay_detected");
        METRICS.counter_inc_labeled("auth_token_refresh_total", "result", "family_revoked");
        return Err(ApiError::UnauthorizedWithCode {
            code: "family_revoked".to_string(),
            message: "Refresh token family revoked due to replay detection".to_string(),
        });
    }

    // 6b. Check expiration
    if expires_at <= now {
        let _ = tx.rollback().await;
        cleanup_lock(state, lock_key, redis_lock_held).await;
        METRICS.counter_inc_labeled("auth_token_refresh_total", "result", "token_expired");
        return Err(ApiError::UnauthorizedWithCode {
            code: "token_expired".to_string(),
            message: "Refresh token has expired".to_string(),
        });
    }

    // Verify user status and auth_version
    let user_row: Option<(i64, String, String)> = match sqlx::query_as(
        "SELECT auth_version, role, status FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.rollback().await;
            cleanup_lock(state, lock_key, redis_lock_held).await;
            return Err(ApiError::Internal(format!("Failed to query user: {:?}", e)));
        }
    };

    let (auth_version, role_str, status_str) = match user_row {
        Some(u) => u,
        None => {
            let _ = tx.rollback().await;
            cleanup_lock(state, lock_key, redis_lock_held).await;
            return Err(ApiError::UnauthorizedWithCode {
                code: "token_invalid".to_string(),
                message: "User does not exist".to_string(),
            });
        }
    };

    if status_str != "ACTIVE" {
        let _ = tx.rollback().await;
        cleanup_lock(state, lock_key, redis_lock_held).await;
        return Err(ApiError::ForbiddenWithCode {
            code: "account_suspended".to_string(),
            message: "Account is suspended or pending verification.".to_string(),
        });
    }

    // 8. Happy path: generate successor refresh token
    let (new_raw, new_hash) = crypto::token::generate_refresh_token();
    let new_token_id = RefreshTokenId::now_v7();
    let new_expires_at = now + chrono::Duration::days(30);

    // Insert new token first so foreign key replaced_by_id is satisfied
    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO refresh_tokens (id, session_id, family_id, token_hash, issued_at, expires_at, revoked_at, replaced_by_id)
        VALUES ($1, $2, $3, $4, $5, $6, NULL, NULL)
        "#
    )
    .bind(new_token_id.into_inner())
    .bind(session_id)
    .bind(family_id)
    .bind(&new_hash)
    .bind(now)
    .bind(new_expires_at)
    .execute(&mut *tx)
    .await {
        let _ = tx.rollback().await;
        cleanup_lock(state, lock_key, redis_lock_held).await;
        return Err(ApiError::Internal(format!("Failed to insert new refresh token: {:?}", e)));
    }

    // Update old token
    if let Err(e) = sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = $1, replaced_by_id = $2 WHERE id = $3"
    )
    .bind(now)
    .bind(new_token_id.into_inner())
    .bind(old_id)
    .execute(&mut *tx)
    .await {
        let _ = tx.rollback().await;
        cleanup_lock(state, lock_key, redis_lock_held).await;
        return Err(ApiError::Internal(format!("Failed to update old refresh token: {:?}", e)));
    }

    // Update session last_seen_at
    let _ = sqlx::query(
        "UPDATE sessions SET last_seen_at = $1 WHERE id = $2"
    )
    .bind(now)
    .bind(session_id)
    .execute(&mut *tx)
    .await;

    // Insert audit log
    let audit_id = AuditLogId::now_v7();
    let _ = sqlx::query(
        r#"
        INSERT INTO audit_log (id, user_id, event, ip_address, user_agent, metadata, created_at)
        VALUES ($1, $2, 'auth.token.refreshed', $3, $4, $5, $6)
        "#
    )
    .bind(audit_id.into_inner())
    .bind(user_id)
    .bind(client_ip)
    .bind(&user_agent)
    .bind(serde_json::json!({
        "session_id": session_id,
        "family_id": family_id,
    }))
    .bind(now)
    .execute(&mut *tx)
    .await;

    if let Err(e) = tx.commit().await {
        cleanup_lock(state, lock_key, redis_lock_held).await;
        return Err(ApiError::Internal(format!("Failed to commit refresh transaction: {:?}", e)));
    }

    // Sign new RS256 access token with sid matching current session
    let now_ts = now.timestamp();
    let claims = crypto::jwt::Claims {
        sub: user_id.to_string(),
        iss: crypto::jwt::DEFAULT_ISSUER.to_string(),
        aud: crypto::jwt::DEFAULT_AUDIENCE.to_string(),
        exp: now_ts + 900,
        iat: now_ts,
        nbf: now_ts,
        jti: Uuid::now_v7(),
        sid: session_id,
        auth_version,
        role: role_str,
        scope: "access".to_string(),
    };

    let access_token = match state.jwt_keys.sign_access_token(&claims) {
        Ok(t) => t,
        Err(e) => {
            cleanup_lock(state, lock_key, redis_lock_held).await;
            return Err(ApiError::Internal(format!("Failed to sign access token: {:?}", e)));
        }
    };

    // Store in Redis grace cache for 15s network retry tolerance (Option A)
    if let Some(mut redis) = state.redis.clone() {
        let grace_key = format!("refresh:grace:{}", token_hash);
        let grace_payload = serde_json::json!({
            "access_token": access_token,
            "raw_refresh": new_raw,
        });
        let _: Result<(), _> = redis.set_ex(&grace_key, grace_payload.to_string(), 15).await;
    }

    cleanup_lock(state, lock_key, redis_lock_held).await;

    METRICS.counter_inc("auth_refresh_rotations_total");
    METRICS.counter_inc_labeled("auth_audit_events_total", "event", "auth.token.refreshed");
    METRICS.counter_inc_labeled("auth_token_refresh_total", "result", "success");

    let cookie_val = format!(
        "__Secure-has_refresh={}; Path=/auth; Max-Age={}; HttpOnly; Secure; SameSite=Lax",
        new_raw,
        30 * 24 * 3600
    );

    let mut response = (StatusCode::OK, Json(RefreshResponse { access_token })).into_response();
    if let Ok(header_val) = axum::http::HeaderValue::from_str(&cookie_val) {
        response.headers_mut().insert(axum::http::header::SET_COOKIE, header_val);
    }

    Ok(response)
}

/// 2. POST /auth/logout
/// Revokes the caller's specific session and all its refresh tokens.
/// Does NOT bump auth_version (other devices remain active).
pub async fn logout(
    auth_user: AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let client_ip = extract_client_ip(&headers);
    let user_agent = extract_user_agent(&headers);
    let now = chrono::Utc::now();
    let sid = auth_user.session_id.into_inner();
    let uid = auth_user.user_id.into_inner();

    let mut tx = state
        .db_pool
        .begin()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to start transaction: {:?}", e)))?;

    // Revoke session
    let _ = sqlx::query(
        "UPDATE sessions SET revoked_at = $1 WHERE id = $2 AND user_id = $3 AND revoked_at IS NULL"
    )
    .bind(now)
    .bind(sid)
    .bind(uid)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to revoke session: {:?}", e)))?;

    // Revoke all refresh tokens for session
    let _ = sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = $1 WHERE session_id = $2 AND revoked_at IS NULL"
    )
    .bind(now)
    .bind(sid)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to revoke refresh tokens: {:?}", e)))?;

    // Insert audit log
    let audit_id = AuditLogId::now_v7();
    let _ = sqlx::query(
        r#"
        INSERT INTO audit_log (id, user_id, event, ip_address, user_agent, metadata, created_at)
        VALUES ($1, $2, 'auth.logout', $3, $4, $5, $6)
        "#
    )
    .bind(audit_id.into_inner())
    .bind(uid)
    .bind(client_ip)
    .bind(&user_agent)
    .bind(serde_json::json!({ "session_id": sid }))
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to write audit log: {:?}", e)))?;

    tx.commit()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to commit logout: {:?}", e)))?;

    METRICS.counter_inc_labeled("auth_audit_events_total", "event", "auth.logout");
    METRICS.counter_inc("auth_logout_total");

    let expired_cookie = "__Secure-has_refresh=; Path=/auth; Max-Age=0; HttpOnly; Secure; SameSite=Lax";
    let mut response = (StatusCode::OK, Json(SuccessResponse { success: true })).into_response();
    if let Ok(header_val) = axum::http::HeaderValue::from_str(expired_cookie) {
        response.headers_mut().insert(axum::http::header::SET_COOKIE, header_val);
    }

    Ok(response)
}

/// 3. POST /auth/logout-all
/// Terminates all sessions across all devices for the calling user, revokes all refresh tokens,
/// and increments users.auth_version within a single transaction. Immediately invalidates
/// the Redis auth_version cache post-commit.
pub async fn logout_all(
    auth_user: AuthUser,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let client_ip = extract_client_ip(&headers);
    let user_agent = extract_user_agent(&headers);
    let now = chrono::Utc::now();
    let uid = auth_user.user_id.into_inner();

    let mut tx = state
        .db_pool
        .begin()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to start transaction: {:?}", e)))?;

    // 1. Bump auth_version in database
    let _ = sqlx::query(
        "UPDATE users SET auth_version = auth_version + 1, updated_at = $1 WHERE id = $2"
    )
    .bind(now)
    .bind(uid)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to bump auth_version: {:?}", e)))?;

    // 2. Revoke all active sessions and count revoked rows
    let revoked_rows = sqlx::query(
        "UPDATE sessions SET revoked_at = $1 WHERE user_id = $2 AND revoked_at IS NULL"
    )
    .bind(now)
    .bind(uid)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to revoke sessions: {:?}", e)))?;

    let sessions_count = revoked_rows.rows_affected();

    // 3. Revoke all refresh tokens for user's sessions
    let _ = sqlx::query(
        r#"
        UPDATE refresh_tokens SET revoked_at = $1
        WHERE session_id IN (SELECT id FROM sessions WHERE user_id = $2)
          AND revoked_at IS NULL
        "#
    )
    .bind(now)
    .bind(uid)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to revoke refresh tokens: {:?}", e)))?;

    // 4. Insert audit log
    let audit_id = AuditLogId::now_v7();
    let _ = sqlx::query(
        r#"
        INSERT INTO audit_log (id, user_id, event, ip_address, user_agent, metadata, created_at)
        VALUES ($1, $2, 'auth.logout_all', $3, $4, $5, $6)
        "#
    )
    .bind(audit_id.into_inner())
    .bind(uid)
    .bind(client_ip)
    .bind(&user_agent)
    .bind(serde_json::json!({ "sessions_revoked": sessions_count }))
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to write audit log: {:?}", e)))?;

    // Commit transaction
    tx.commit()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to commit logout-all transaction: {:?}", e)))?;

    // 5. Post-commit synchronous cache invalidation (Phase 3 §1.g & Decision l)
    if let Some(mut redis) = state.redis.clone() {
        let auth_version_key = format!("auth_version:{}", uid);
        let _: Result<(), _> = redis.del(&auth_version_key).await;
    }

    METRICS.counter_inc_labeled("auth_audit_events_total", "event", "auth.logout_all");
    METRICS.counter_inc("auth_logout_all_total");

    let expired_cookie = "__Secure-has_refresh=; Path=/auth; Max-Age=0; HttpOnly; Secure; SameSite=Lax";
    let mut response = (StatusCode::OK, Json(LogoutAllResponse {
        success: true,
        sessions_revoked: sessions_count,
    })).into_response();

    if let Ok(header_val) = axum::http::HeaderValue::from_str(expired_cookie) {
        response.headers_mut().insert(axum::http::header::SET_COOKIE, header_val);
    }

    Ok(response)
}

/// 4. GET /auth/sessions
/// Lists active sessions for the calling user. Marks current session and masks IP addresses.
/// Caps at 50 sessions without cursor pagination (Phase 5 Decision f).
pub async fn list_sessions(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<SessionDto>>, ApiError> {
    let now = chrono::Utc::now();
    let uid = auth_user.user_id.into_inner();
    let current_sid = auth_user.session_id.into_inner();

    let rows: Vec<(Uuid, Option<String>, Option<String>, Option<IpAddr>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        r#"
        SELECT id, device_fingerprint, user_agent, ip_address, created_at, last_seen_at, expires_at
        FROM sessions
        WHERE user_id = $1
          AND revoked_at IS NULL
          AND expires_at > $2
        ORDER BY last_seen_at DESC
        LIMIT 50
        "#
    )
    .bind(uid)
    .bind(now)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error querying sessions: {:?}", e)))?;

    let sessions = rows
        .into_iter()
        .map(|(id, device_fingerprint, ua, ip_opt, created_at, last_seen_at, expires_at)| {
            let ip_str = ip_opt
                .map(|ip| mask_ip(&ip))
                .unwrap_or_else(|| "127.0.0.1".to_string());
            SessionDto {
                id,
                current: (id == current_sid),
                device_fingerprint,
                user_agent: ua.unwrap_or_else(|| "unknown".to_string()),
                ip_address: ip_str,
                created_at,
                last_seen_at,
                expires_at,
            }
        })
        .collect();

    Ok(Json(sessions))
}

/// 5. DELETE /auth/sessions/{id}
/// Terminates a specific remote session belonging to the caller.
/// If the target session does not exist, belongs to another user, or is already revoked,
/// returns uniform HTTP 404 not_found to prevent identifier enumeration (Decision g).
pub async fn delete_session(
    auth_user: AuthUser,
    Path(target_id_str): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let target_id = match Uuid::from_str(&target_id_str) {
        Ok(u) => u,
        Err(_) => {
            return Err(ApiError::BadRequestWithCode {
                code: "validation_error".to_string(),
                message: "Invalid session ID format".to_string(),
            });
        }
    };

    let client_ip = extract_client_ip(&headers);
    let user_agent = extract_user_agent(&headers);
    let now = chrono::Utc::now();
    let uid = auth_user.user_id.into_inner();
    let current_sid = auth_user.session_id.into_inner();

    let mut tx = state
        .db_pool
        .begin()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to start transaction: {:?}", e)))?;

    let session_row: Option<(Uuid, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        "SELECT user_id, revoked_at FROM sessions WHERE id = $1 FOR UPDATE"
    )
    .bind(target_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Database error checking session: {:?}", e)))?;

    match session_row {
        Some((owner_id, revoked_at)) => {
            // Anti-enumeration: if not owned by caller or already revoked, return 404
            if owner_id != uid || revoked_at.is_some() {
                let _ = tx.rollback().await;
                return Err(ApiError::NotFoundWithCode {
                    code: "not_found".to_string(),
                    message: "Session not found".to_string(),
                });
            }
        }
        None => {
            let _ = tx.rollback().await;
            return Err(ApiError::NotFoundWithCode {
                code: "not_found".to_string(),
                message: "Session not found".to_string(),
            });
        }
    }

    // Revoke session
    let _ = sqlx::query(
        "UPDATE sessions SET revoked_at = $1 WHERE id = $2"
    )
    .bind(now)
    .bind(target_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to revoke session: {:?}", e)))?;

    // Revoke all refresh tokens for target session
    let _ = sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = $1 WHERE session_id = $2 AND revoked_at IS NULL"
    )
    .bind(now)
    .bind(target_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to revoke refresh tokens: {:?}", e)))?;

    // Insert audit log
    let audit_id = AuditLogId::now_v7();
    let _ = sqlx::query(
        r#"
        INSERT INTO audit_log (id, user_id, event, ip_address, user_agent, metadata, created_at)
        VALUES ($1, $2, 'auth.session.revoked_remote', $3, $4, $5, $6)
        "#
    )
    .bind(audit_id.into_inner())
    .bind(uid)
    .bind(client_ip)
    .bind(&user_agent)
    .bind(serde_json::json!({ "target_session_id": target_id }))
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to write audit log: {:?}", e)))?;

    tx.commit()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to commit delete session transaction: {:?}", e)))?;

    METRICS.counter_inc_labeled("auth_audit_events_total", "event", "auth.session.revoked_remote");
    METRICS.counter_inc("auth_session_deleted_total");

    let mut response = (StatusCode::OK, Json(SuccessResponse { success: true })).into_response();

    // If caller deleted their own current session, clear the cookie
    if target_id == current_sid {
        let expired_cookie = "__Secure-has_refresh=; Path=/auth; Max-Age=0; HttpOnly; Secure; SameSite=Lax";
        if let Ok(header_val) = axum::http::HeaderValue::from_str(expired_cookie) {
            response.headers_mut().insert(axum::http::header::SET_COOKIE, header_val);
        }
    }

    Ok(response)
}
