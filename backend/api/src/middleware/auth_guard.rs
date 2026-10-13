// backend/api/src/middleware/auth_guard.rs

use std::ops::Deref;
use std::sync::Arc;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use redis::AsyncCommands;
use serde_json::json;
use uuid::Uuid;

use domain::models::identifiers::{SessionId, UserId};
use domain::models::user::UserRole;
use crate::metrics::METRICS;
use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: UserId,
    pub role: UserRole,
    #[allow(dead_code)]
    pub session_id: SessionId,
    #[allow(dead_code)]
    pub auth_version: i64,
    #[allow(dead_code)]
    pub jti: Uuid,
}

#[derive(Debug, Clone)]
pub struct AuthRejection {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}

impl AuthRejection {
    pub fn invalid_token(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "invalid_token",
            message: msg.into(),
        }
    }

    pub fn token_expired() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "token_expired",
            message: "Token has expired or is not yet valid".to_string(),
        }
    }

    pub fn token_revoked() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "token_revoked",
            message: "Token has been revoked".to_string(),
        }
    }

    pub fn account_suspended() -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: "account_suspended",
            message: "Account is suspended or inactive".to_string(),
        }
    }

    pub fn permission_denied() -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: "permission_denied",
            message: "Insufficient permissions for this resource".to_string(),
        }
    }

    pub fn internal_error() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: "Internal authentication error".to_string(),
        }
    }
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error": {
                "code": self.code,
                "message": self.message,
            }
        }));
        (self.status, body).into_response()
    }
}

/// Core authentication pipeline executing steps 1–12:
/// 1. Extract Bearer token from Authorization header.
/// 2. If missing/invalid -> 401 invalid_token.
/// 3. Parse header. Missing kid -> 401 invalid_token.
/// 4. Unknown kid / signature verification via JwtKeys::verify_access_token.
/// 5. Verification failure -> 401 invalid_token (or 401 token_expired if expired/nbf).
/// 6. Check claims.scope == "access" -> 401 invalid_token if not.
/// 7. Read auth_version:{user_id} and user_status:{user_id} from Redis.
///    - On miss or Redis error: SELECT auth_version, status FROM users WHERE id = $1.
///    - Populate Redis cache on miss.
/// 8. If claims.auth_version != live -> 401 token_revoked.
/// 9. If live status != ACTIVE -> 403 account_suspended.
/// 10. Memoize AuthenticatedUser in request extensions.
pub async fn authenticate_request(
    parts: &mut Parts,
    state: &AppState,
) -> Result<AuthenticatedUser, AuthRejection> {
    // Check if user has already been authenticated in this request (extension cache)
    if let Some(user) = parts.extensions.get::<Arc<AuthenticatedUser>>() {
        return Ok((**user).clone());
    }

    // 1. Extract token from Authorization: Bearer <jwt> header
    let auth_header = match parts.headers.get(AUTHORIZATION) {
        Some(h) => h,
        None => {
            METRICS.counter_inc_labeled("auth_middleware_rejection_total", "reason", "invalid_token");
            return Err(AuthRejection::invalid_token("Missing Authorization header"));
        }
    };

    let auth_str = match auth_header.to_str() {
        Ok(s) => s,
        Err(_) => {
            METRICS.counter_inc_labeled("auth_middleware_rejection_total", "reason", "invalid_token");
            return Err(AuthRejection::invalid_token("Invalid header encoding"));
        }
    };

    if !auth_str.starts_with("Bearer ") {
        METRICS.counter_inc_labeled("auth_middleware_rejection_total", "reason", "invalid_token");
        return Err(AuthRejection::invalid_token("Authorization scheme must be Bearer"));
    }

    let token = auth_str[7..].trim();
    if token.is_empty() {
        METRICS.counter_inc_labeled("auth_middleware_rejection_total", "reason", "invalid_token");
        return Err(AuthRejection::invalid_token("Empty bearer token"));
    }

    // 2. Decode header to enforce kid presence and RS256 alg
    let header = match jsonwebtoken::decode_header(token) {
        Ok(h) => h,
        Err(_) => {
            METRICS.counter_inc_labeled("auth_middleware_rejection_total", "reason", "invalid_token");
            return Err(AuthRejection::invalid_token("Malformed JWT header"));
        }
    };

    if header.kid.is_none() {
        METRICS.counter_inc_labeled("auth_middleware_rejection_total", "reason", "invalid_token");
        return Err(AuthRejection::invalid_token("Missing kid in token header"));
    }

    // 3. Verify RS256 signature and standard claims (exp, nbf, iss, aud) via JwtKeys
    let claims = match state.jwt_keys.verify_access_token(token) {
        Ok(c) => c,
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("expired") || err_msg.contains("not yet valid") || err_msg.contains("nbf") {
                METRICS.counter_inc_labeled("auth_middleware_rejection_total", "reason", "token_expired");
                return Err(AuthRejection::token_expired());
            } else {
                METRICS.counter_inc_labeled("auth_middleware_rejection_total", "reason", "invalid_token");
                return Err(AuthRejection::invalid_token("Invalid token signature or key"));
            }
        }
    };

    // 4. Enforce scope == "access"
    if claims.scope != "access" {
        METRICS.counter_inc_labeled("auth_middleware_rejection_total", "reason", "invalid_token");
        return Err(AuthRejection::invalid_token("Token scope must be 'access'"));
    }

    let user_uuid = match Uuid::parse_str(&claims.sub) {
        Ok(u) => u,
        Err(_) => {
            METRICS.counter_inc_labeled("auth_middleware_rejection_total", "reason", "invalid_token");
            return Err(AuthRejection::invalid_token("Invalid subject in claims"));
        }
    };
    let user_id = UserId(user_uuid);

    // 5. Read auth_version:{user_id} and user_status:{user_id} from Redis
    let auth_version_key = format!("auth_version:{}", user_id.into_inner());
    let user_status_key = format!("user_status:{}", user_id.into_inner());

    let mut live_auth_version: Option<i64> = None;
    let mut live_status: Option<String> = None;

    if let Some(mut redis) = state.redis.clone() {
        let v_res: Result<Option<i64>, _> = redis.get(&auth_version_key).await;
        let s_res: Result<Option<String>, _> = redis.get(&user_status_key).await;

        if let (Ok(Some(v)), Ok(Some(s))) = (v_res, s_res) {
            live_auth_version = Some(v);
            live_status = Some(s);
        }
    }

    // If cache miss or Redis unavailable: fall back to PostgreSQL
    if live_auth_version.is_none() || live_status.is_none() {
        let row: Option<(i64, String)> = sqlx::query_as(
            "SELECT auth_version, status FROM users WHERE id = $1",
        )
        .bind(user_id.into_inner())
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Database query error during auth check: {:?}", e);
            AuthRejection::internal_error()
        })?;

        match row {
            Some((db_version, db_status)) => {
                live_auth_version = Some(db_version);
                live_status = Some(db_status.clone());

                // Best-effort cache population (300s TTL)
                if let Some(mut redis) = state.redis.clone() {
                    let _: Result<(), _> = redis.set_ex(&auth_version_key, db_version, 300).await;
                    let _: Result<(), _> = redis.set_ex(&user_status_key, db_status, 300).await;
                }
            }
            None => {
                METRICS.counter_inc_labeled("auth_middleware_rejection_total", "reason", "invalid_token");
                return Err(AuthRejection::invalid_token("User does not exist"));
            }
        }
    }

    // 6. Check auth_version mismatch
    if claims.auth_version != live_auth_version.unwrap() {
        METRICS.counter_inc_labeled("auth_middleware_rejection_total", "reason", "token_revoked");
        return Err(AuthRejection::token_revoked());
    }

    // 7. Check user status == ACTIVE
    let status_str = live_status.unwrap();
    if status_str != "ACTIVE" {
        METRICS.counter_inc_labeled("auth_middleware_rejection_total", "reason", "account_suspended");
        return Err(AuthRejection::account_suspended());
    }

    let role = UserRole::try_from(claims.role.as_str()).map_err(|_| {
        METRICS.counter_inc_labeled("auth_middleware_rejection_total", "reason", "invalid_token");
        AuthRejection::invalid_token("Unknown role in token claims")
    })?;

    let session_id = SessionId(claims.jti);

    let authenticated_user = AuthenticatedUser {
        user_id,
        role,
        session_id,
        auth_version: claims.auth_version,
        jti: claims.jti,
    };

    // Attach to request extensions for downstream extractors/handlers
    let user_arc = Arc::new(authenticated_user.clone());
    parts.extensions.insert(user_arc);

    Ok(authenticated_user)
}

// ---------------------------------------------------------------------------
// Per-Role Extractors
// ---------------------------------------------------------------------------

/// General authenticated user extractor (any valid role).
#[derive(Debug, Clone)]
pub struct AuthUser(pub AuthenticatedUser);

impl Deref for AuthUser {
    type Target = AuthenticatedUser;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let user = authenticate_request(parts, &app_state).await?;
        Ok(AuthUser(user))
    }
}

/// Extractor requiring at least Customer role (Customer, Moderator, or Admin).
#[derive(Debug, Clone)]
pub struct RequireCustomer(pub AuthenticatedUser);

impl Deref for RequireCustomer {
    type Target = AuthenticatedUser;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> FromRequestParts<S> for RequireCustomer
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let user = authenticate_request(parts, &app_state).await?;
        match user.role {
            UserRole::Customer | UserRole::Moderator | UserRole::Admin => Ok(RequireCustomer(user)),
        }
    }
}

/// Extractor requiring Moderator or Admin role.
#[derive(Debug, Clone)]
pub struct RequireModerator(pub AuthenticatedUser);

impl Deref for RequireModerator {
    type Target = AuthenticatedUser;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> FromRequestParts<S> for RequireModerator
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let user = authenticate_request(parts, &app_state).await?;
        match user.role {
            UserRole::Moderator | UserRole::Admin => Ok(RequireModerator(user)),
            UserRole::Customer => {
                METRICS.counter_inc_labeled("auth_middleware_rejection_total", "reason", "permission_denied");
                Err(AuthRejection::permission_denied())
            }
        }
    }
}

/// Extractor requiring Admin role.
#[derive(Debug, Clone)]
pub struct RequireAdmin(pub AuthenticatedUser);

impl Deref for RequireAdmin {
    type Target = AuthenticatedUser;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> FromRequestParts<S> for RequireAdmin
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let user = authenticate_request(parts, &app_state).await?;
        match user.role {
            UserRole::Admin => Ok(RequireAdmin(user)),
            UserRole::Customer | UserRole::Moderator => {
                METRICS.counter_inc_labeled("auth_middleware_rejection_total", "reason", "permission_denied");
                Err(AuthRejection::permission_denied())
            }
        }
    }
}
