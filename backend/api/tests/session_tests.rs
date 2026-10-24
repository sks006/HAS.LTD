// backend/api/tests/session_tests.rs

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use http_body_util::BodyExt;
use redis::AsyncCommands;
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use api::{
    routes::build_router,
    state::AppState,
};
use crypto::jwt::{Claims, DEFAULT_AUDIENCE, DEFAULT_ISSUER};
use domain::models::identifiers::{CredentialId, RefreshTokenId, SessionId, UserId};
use domain::models::user::{UserRole, UserStatus};

struct TestHarness {
    state: AppState,
}

impl TestHarness {
    async fn new() -> Self {
        dotenvy::dotenv().ok();
        let state = AppState::new().await.expect("Failed to initialize AppState");
        Self { state }
    }

    fn router(&self) -> axum::Router {
        build_router(self.state.clone())
    }

    async fn create_user_with_session(
        &self,
        role: UserRole,
        status: UserStatus,
        auth_version: i64,
    ) -> (UserId, SessionId, RefreshTokenId, Uuid, String, String) {
        let user_id = UserId::now_v7();
        let email = format!("test_{}@has.ltd", Uuid::now_v7());
        let password = "Password123!".to_string();
        let password_hash = crypto::password::hash_password(&password).unwrap();
        let now = chrono::Utc::now();

        // 1. Insert user
        sqlx::query(
            r#"
            INSERT INTO users (id, email, email_normalized, name, role, status, auth_version, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
            "#,
        )
        .bind(user_id.into_inner())
        .bind(&email)
        .bind(&email)
        .bind("Test Session User")
        .bind(role.as_str())
        .bind(status.as_str())
        .bind(auth_version)
        .bind(now)
        .execute(&self.state.db_pool)
        .await
        .unwrap();

        // 2. Insert credentials
        let cred_id = CredentialId::now_v7();
        sqlx::query(
            r#"
            INSERT INTO credentials (id, user_id, kind, provider, provider_subject, secret_hash, created_at, updated_at)
            VALUES ($1, $2, 'password'::credential_kind_enum, 'local', NULL, $3, $4, $4)
            "#,
        )
        .bind(cred_id.into_inner())
        .bind(user_id.into_inner())
        .bind(&password_hash)
        .bind(now)
        .execute(&self.state.db_pool)
        .await
        .unwrap();

        // 3. Insert session
        let session_id = SessionId::now_v7();
        sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, user_agent, ip_address, created_at, last_seen_at, expires_at, revoked_at)
            VALUES ($1, $2, 'test-agent/1.0', '127.0.0.1'::inet, $3, $3, $4, NULL)
            "#,
        )
        .bind(session_id.into_inner())
        .bind(user_id.into_inner())
        .bind(now)
        .bind(now + chrono::Duration::days(30))
        .execute(&self.state.db_pool)
        .await
        .unwrap();

        // 4. Insert refresh token
        let refresh_id = RefreshTokenId::now_v7();
        let family_id = Uuid::new_v4();
        let (raw_token, token_hash) = crypto::token::generate_refresh_token();
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, session_id, family_id, token_hash, issued_at, expires_at, revoked_at)
            VALUES ($1, $2, $3, $4, $5, $6, NULL)
            "#,
        )
        .bind(refresh_id.into_inner())
        .bind(session_id.into_inner())
        .bind(family_id)
        .bind(&token_hash)
        .bind(now)
        .bind(now + chrono::Duration::days(30))
        .execute(&self.state.db_pool)
        .await
        .unwrap();

        // 5. Sign access token
        let access_token = self.issue_token(user_id, role, auth_version, "access", session_id);

        (user_id, session_id, refresh_id, family_id, raw_token, access_token)
    }

    async fn add_session_for_user(
        &self,
        user_id: UserId,
        user_agent: &str,
        ip: &str,
    ) -> (SessionId, RefreshTokenId, Uuid, String, String) {
        let session_id = SessionId::now_v7();
        let now = chrono::Utc::now();
        sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, user_agent, ip_address, created_at, last_seen_at, expires_at, revoked_at)
            VALUES ($1, $2, $3, $4::inet, $5, $5, $6, NULL)
            "#,
        )
        .bind(session_id.into_inner())
        .bind(user_id.into_inner())
        .bind(user_agent)
        .bind(ip)
        .bind(now)
        .bind(now + chrono::Duration::days(30))
        .execute(&self.state.db_pool)
        .await
        .unwrap();

        let refresh_id = RefreshTokenId::now_v7();
        let family_id = Uuid::new_v4();
        let (raw_token, token_hash) = crypto::token::generate_refresh_token();
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, session_id, family_id, token_hash, issued_at, expires_at, revoked_at)
            VALUES ($1, $2, $3, $4, $5, $6, NULL)
            "#,
        )
        .bind(refresh_id.into_inner())
        .bind(session_id.into_inner())
        .bind(family_id)
        .bind(&token_hash)
        .bind(now)
        .bind(now + chrono::Duration::days(30))
        .execute(&self.state.db_pool)
        .await
        .unwrap();

        let access_token = self.issue_token(user_id, UserRole::Customer, 1, "access", session_id);
        (session_id, refresh_id, family_id, raw_token, access_token)
    }

    fn issue_token(
        &self,
        user_id: UserId,
        role: UserRole,
        auth_version: i64,
        scope: &str,
        sid: SessionId,
    ) -> String {
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: user_id.into_inner().to_string(),
            iss: DEFAULT_ISSUER.to_string(),
            aud: DEFAULT_AUDIENCE.to_string(),
            exp: now + 3600,
            iat: now,
            nbf: now,
            jti: Uuid::now_v7(),
            sid: sid.into_inner(),
            auth_version,
            role: role.to_string(),
            scope: scope.to_string(),
        };
        self.state.jwt_keys.sign_access_token(&claims).unwrap()
    }
}

async fn response_to_json(resp: axum::response::Response) -> (StatusCode, Value) {
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
    (status, json)
}

fn extract_refresh_cookie_value(resp: &axum::response::Response) -> Option<String> {
    for val in resp.headers().get_all(header::SET_COOKIE) {
        if let Ok(s) = val.to_str() {
            if let Some(cookie_part) = s.split(';').next() {
                if let Some(token) = cookie_part.strip_prefix("__Secure-has_refresh=") {
                    return Some(token.to_string());
                }
            }
        }
    }
    None
}

fn is_refresh_cookie_cleared(resp: &axum::response::Response) -> bool {
    for val in resp.headers().get_all(header::SET_COOKIE) {
        if let Ok(s) = val.to_str() {
            if s.contains("__Secure-has_refresh=") && (s.contains("Max-Age=0") || s.contains("expires=Thu, 01 Jan 1970")) {
                return true;
            }
        }
    }
    false
}

// ============================================================================
// 1. POST /auth/refresh Tests
// ============================================================================

#[tokio::test]
async fn test_refresh_happy_path() {
    let harness = TestHarness::new().await;
    let (_uid, sid, old_refresh_id, family_id, raw_token, _) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;

    let req = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::COOKIE, format!("__Secure-has_refresh={}", raw_token))
        .body(Body::empty())
        .unwrap();

    let resp = harness.router().oneshot(req).await.unwrap();
    let status = resp.status();
    let headers = resp.headers().clone();
    let (_, body) = response_to_json(resp).await;
    println!("refresh resp: status={}, body={:?}", status, body);
    assert_eq!(status, StatusCode::OK);

    // Assert Set-Cookie attributes
    let set_cookie = headers.get(header::SET_COOKIE).unwrap().to_str().unwrap();
    assert!(set_cookie.contains("__Secure-has_refresh="));
    assert!(set_cookie.contains("Path=/auth"));
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("Secure"));
    assert!(set_cookie.contains("SameSite=Lax"));
    assert!(set_cookie.contains("Max-Age=2592000"));

    let mut dummy_resp = axum::response::Response::new(Body::empty());
    *dummy_resp.headers_mut() = headers;
    let new_raw_token = extract_refresh_cookie_value(&dummy_resp).unwrap();
    assert_ne!(new_raw_token, raw_token);

    let new_access_token = body["access_token"].as_str().unwrap();

    // Verify claims of new access token
    let claims = harness.state.jwt_keys.verify_access_token(new_access_token).unwrap();
    assert_eq!(claims.sid, sid.into_inner());
    assert_eq!(claims.scope, "access");

    // Verify DB: old token revoked and replaced_by_id set
    let (revoked_at, replaced_by_id): (Option<chrono::DateTime<chrono::Utc>>, Option<Uuid>) =
        sqlx::query_as("SELECT revoked_at, replaced_by_id FROM refresh_tokens WHERE id = $1")
            .bind(old_refresh_id.into_inner())
            .fetch_one(&harness.state.db_pool)
            .await
            .unwrap();
    assert!(revoked_at.is_some());
    assert!(replaced_by_id.is_some());

    // Verify DB: new token inserted with matching family_id and distinct hash
    let new_token_hash = crypto::token::sha256_hex(&URL_SAFE_NO_PAD.decode(&new_raw_token).unwrap());
    let (stored_family_id, stored_hash, new_revoked_at): (Uuid, String, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as("SELECT family_id, token_hash, revoked_at FROM refresh_tokens WHERE id = $1")
            .bind(replaced_by_id.unwrap())
            .fetch_one(&harness.state.db_pool)
            .await
            .unwrap();
    assert_eq!(stored_family_id, family_id);
    assert_eq!(stored_hash, new_token_hash);
    assert!(new_revoked_at.is_none());

    // Verify audit log
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE event = 'auth.token.refreshed' AND metadata->>'session_id' = $1"
    )
    .bind(sid.into_inner().to_string())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert!(count >= 1);
}

#[tokio::test]
async fn test_refresh_missing_cookie_returns_401() {
    let harness = TestHarness::new().await;
    let req = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .body(Body::empty())
        .unwrap();

    let resp = harness.router().oneshot(req).await.unwrap();
    let (status, body) = response_to_json(resp).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_token");
}

#[tokio::test]
async fn test_refresh_malformed_cookie_returns_401() {
    let harness = TestHarness::new().await;
    let req = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::COOKIE, "__Secure-has_refresh=invalid_base64_not_32_bytes!")
        .body(Body::empty())
        .unwrap();

    let resp = harness.router().oneshot(req).await.unwrap();
    let (status, body) = response_to_json(resp).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "token_invalid");
}

#[tokio::test]
async fn test_refresh_unknown_token_returns_401() {
    let harness = TestHarness::new().await;
    let (random_token, _) = crypto::token::generate_refresh_token();

    let req = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::COOKIE, format!("__Secure-has_refresh={}", random_token))
        .body(Body::empty())
        .unwrap();

    let resp = harness.router().oneshot(req).await.unwrap();
    let (status, body) = response_to_json(resp).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "token_invalid");
}

#[tokio::test]
async fn test_refresh_expired_token_returns_401() {
    let harness = TestHarness::new().await;
    let (_uid, _sid, refresh_id, _fam, raw_token, _) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;

    // Expire token in DB
    sqlx::query("UPDATE refresh_tokens SET expires_at = now() - INTERVAL '1 day' WHERE id = $1")
        .bind(refresh_id.into_inner())
        .execute(&harness.state.db_pool)
        .await
        .unwrap();

    let req = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::COOKIE, format!("__Secure-has_refresh={}", raw_token))
        .body(Body::empty())
        .unwrap();

    let resp = harness.router().oneshot(req).await.unwrap();
    let (status, body) = response_to_json(resp).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "token_expired");
}

#[tokio::test]
async fn test_refresh_revoked_session_returns_401() {
    let harness = TestHarness::new().await;
    let (_uid, sid, _refresh_id, _fam, raw_token, _) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;

    // Revoke session in DB
    sqlx::query("UPDATE sessions SET revoked_at = now() WHERE id = $1")
        .bind(sid.into_inner())
        .execute(&harness.state.db_pool)
        .await
        .unwrap();

    let req = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::COOKIE, format!("__Secure-has_refresh={}", raw_token))
        .body(Body::empty())
        .unwrap();

    let resp = harness.router().oneshot(req).await.unwrap();
    let (status, body) = response_to_json(resp).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "token_revoked");
}

#[tokio::test]
async fn test_refresh_grace_window_success() {
    let harness = TestHarness::new().await;
    let (_uid, _sid, _refresh_id, _fam, raw_token, _) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;

    // First refresh: succeeds
    let req1 = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::COOKIE, format!("__Secure-has_refresh={}", raw_token))
        .body(Body::empty())
        .unwrap();

    let resp1 = harness.router().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);
    let cookie1 = extract_refresh_cookie_value(&resp1).unwrap();
    let (_, body1) = response_to_json(resp1).await;
    let access1 = body1["access_token"].as_str().unwrap();

    // Second refresh with SAME old token (within 15s grace window)
    let req2 = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::COOKIE, format!("__Secure-has_refresh={}", raw_token))
        .body(Body::empty())
        .unwrap();

    let resp2 = harness.router().oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);
    let cookie2 = extract_refresh_cookie_value(&resp2).unwrap();
    let (_, body2) = response_to_json(resp2).await;
    let access2 = body2["access_token"].as_str().unwrap();

    // Idempotent: returns identical access token and identical refresh cookie
    assert_eq!(access1, access2);
    assert_eq!(cookie1, cookie2);
}

#[tokio::test]
async fn test_refresh_replay_detection_after_grace() {
    let harness = TestHarness::new().await;
    let (uid, sid, old_refresh_id, family_id, raw_token, _) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;

    // First refresh: succeeds
    let req1 = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::COOKIE, format!("__Secure-has_refresh={}", raw_token))
        .body(Body::empty())
        .unwrap();
    let resp1 = harness.router().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);

    // Delete Redis grace key to simulate cache eviction/expiration
    let raw_bytes = URL_SAFE_NO_PAD.decode(&raw_token).unwrap();
    let old_token_hash = crypto::token::sha256_hex(&raw_bytes);
    if let Some(mut redis_conn) = harness.state.redis.clone() {
        let grace_key = format!("refresh:grace:{}", old_token_hash);
        let _: () = redis_conn.del(grace_key).await.unwrap();
    }

    // Set revoked_at to 20 seconds ago (> 15s grace period)
    sqlx::query("UPDATE refresh_tokens SET revoked_at = now() - INTERVAL '20 seconds' WHERE id = $1")
        .bind(old_refresh_id.into_inner())
        .execute(&harness.state.db_pool)
        .await
        .unwrap();

    // Present the revoked token again -> Replay detection!
    let req2 = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::COOKIE, format!("__Secure-has_refresh={}", raw_token))
        .body(Body::empty())
        .unwrap();

    let resp2 = harness.router().oneshot(req2).await.unwrap();
    let (status, body) = response_to_json(resp2).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "family_revoked");

    // Verify DB: entire family revoked
    let unrevoked_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM refresh_tokens WHERE family_id = $1 AND revoked_at IS NULL"
    )
    .bind(family_id)
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert_eq!(unrevoked_count, 0);

    // Verify DB: session revoked
    let session_revoked_at: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT revoked_at FROM sessions WHERE id = $1")
            .bind(sid.into_inner())
            .fetch_one(&harness.state.db_pool)
            .await
            .unwrap();
    assert!(session_revoked_at.is_some());

    // Verify DB: user auth_version incremented to 2
    let auth_version: i64 = sqlx::query_scalar("SELECT auth_version FROM users WHERE id = $1")
        .bind(uid.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    assert_eq!(auth_version, 2);

    // Verify audit row: auth.session.replay_detected
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE event = 'auth.session.replay_detected' AND user_id = $1"
    )
    .bind(uid.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert_eq!(audit_count, 1);
}

#[tokio::test]
async fn test_refresh_concurrent_same_token() {
    let harness = TestHarness::new().await;
    let (_uid, _sid, _refresh_id, _fam, raw_token, _) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;

    let app1 = harness.router();
    let app2 = harness.router();

    let token1 = raw_token.clone();
    let token2 = raw_token.clone();

    let h1 = tokio::spawn(async move {
        let req = Request::builder()
            .method("POST")
            .uri("/auth/refresh")
            .header(header::COOKIE, format!("__Secure-has_refresh={}", token1))
            .body(Body::empty())
            .unwrap();
        app1.oneshot(req).await.unwrap()
    });

    let h2 = tokio::spawn(async move {
        let req = Request::builder()
            .method("POST")
            .uri("/auth/refresh")
            .header(header::COOKIE, format!("__Secure-has_refresh={}", token2))
            .body(Body::empty())
            .unwrap();
        app2.oneshot(req).await.unwrap()
    });

    let (res1, res2) = tokio::join!(h1, h2);
    let resp1 = res1.unwrap();
    let resp2 = res2.unwrap();

    let s1 = resp1.status();
    let s2 = resp2.status();

    // Exactly one must be 200, and the other must be either 409 refresh_in_progress or 200 (grace window).
    // They must NEVER produce two differing successors.
    let is_valid_outcome = (s1 == StatusCode::OK && (s2 == StatusCode::CONFLICT || s2 == StatusCode::OK))
        || (s2 == StatusCode::OK && (s1 == StatusCode::CONFLICT || s1 == StatusCode::OK));
    assert!(is_valid_outcome, "Unexpected concurrent statuses: s1={}, s2={}", s1, s2);

    if s1 == StatusCode::OK && s2 == StatusCode::OK {
        let (_, b1) = response_to_json(resp1).await;
        let (_, b2) = response_to_json(resp2).await;
        assert_eq!(b1["access_token"], b2["access_token"]);
    }
}

#[tokio::test]
async fn test_refresh_with_redis_down_fallback() {
    let harness = TestHarness::new().await;
    let (_uid, _sid, _refresh_id, _fam, raw_token, _) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;

    // Construct degraded AppState with redis = None
    let mut degraded_state = harness.state.clone();
    degraded_state.redis = None;
    let degraded_app = build_router(degraded_state);

    let req = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::COOKIE, format!("__Secure-has_refresh={}", raw_token))
        .body(Body::empty())
        .unwrap();

    let resp = degraded_app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let (status, body) = response_to_json(resp).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["access_token"].is_string());
}

#[tokio::test]
async fn test_refresh_grace_window_redis_down_does_not_family_revoke() {
    let harness = TestHarness::new().await;
    let (uid, _sid, old_refresh_id, _family_id, raw_token, _) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;

    // 1. Initial refresh succeeds (200), issuing successor and revoking old_token with replaced_by_id
    let req1 = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::COOKIE, format!("__Secure-has_refresh={}", raw_token))
        .body(Body::empty())
        .unwrap();

    let resp1 = harness.router().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);

    // Fetch successor token ID
    let replaced_by_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT replaced_by_id FROM refresh_tokens WHERE id = $1"
    )
    .bind(old_refresh_id.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert!(replaced_by_id.is_some());
    let successor_id = replaced_by_id.unwrap();

    // 2. Simulate Redis down during grace retry within 15s
    let mut degraded_state = harness.state.clone();
    degraded_state.redis = None;
    let degraded_app = build_router(degraded_state);

    let req2 = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::COOKIE, format!("__Secure-has_refresh={}", raw_token))
        .body(Body::empty())
        .unwrap();

    let resp2 = degraded_app.oneshot(req2).await.unwrap();
    let (status, body) = response_to_json(resp2).await;

    // Assertion 1: returns 401 token_invalid (NOT family_revoked)
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "token_invalid");

    // Assertion 2: successor token remains valid (revoked_at IS NULL)
    let successor_revoked_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        "SELECT revoked_at FROM refresh_tokens WHERE id = $1"
    )
    .bind(successor_id)
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert!(successor_revoked_at.is_none(), "Successor token was prematurely revoked");

    // Assertion 3: users.auth_version was NOT incremented (remains 1)
    let auth_version: i64 = sqlx::query_scalar(
        "SELECT auth_version FROM users WHERE id = $1"
    )
    .bind(uid.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert_eq!(auth_version, 1);

    // Assertion 4: audit_log contains no 'auth.session.replay_detected' row
    let replay_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE event = 'auth.session.replay_detected' AND user_id = $1"
    )
    .bind(uid.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert_eq!(replay_audit_count, 0);
}

// ============================================================================
// 2. POST /auth/logout Tests
// ============================================================================

#[tokio::test]
async fn test_logout_revokes_calling_session_and_clears_cookie() {
    let harness = TestHarness::new().await;
    let (uid, sid, _ref_id, _fam, _raw, access_token) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;

    // Add a second session on another device for the same user
    let (sid2, _, _, _, _) = harness.add_session_for_user(uid, "device-2", "192.168.1.1").await;

    let req = Request::builder()
        .method("POST")
        .uri("/auth/logout")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();

    let resp = harness.router().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(is_refresh_cookie_cleared(&resp));

    let (status, body) = response_to_json(resp).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);

    // Verify DB: calling session revoked
    let s1_revoked: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT revoked_at FROM sessions WHERE id = $1")
            .bind(sid.into_inner())
            .fetch_one(&harness.state.db_pool)
            .await
            .unwrap();
    assert!(s1_revoked.is_some());

    // Verify DB: calling session's refresh tokens revoked
    let unrevoked_rt1: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM refresh_tokens WHERE session_id = $1 AND revoked_at IS NULL"
    )
    .bind(sid.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert_eq!(unrevoked_rt1, 0);

    // Verify DB: second session is still active
    let s2_revoked: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT revoked_at FROM sessions WHERE id = $1")
            .bind(sid2.into_inner())
            .fetch_one(&harness.state.db_pool)
            .await
            .unwrap();
    assert!(s2_revoked.is_none());

    // Verify DB: user auth_version was NOT bumped on single logout
    let auth_version: i64 = sqlx::query_scalar("SELECT auth_version FROM users WHERE id = $1")
        .bind(uid.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    assert_eq!(auth_version, 1);

    // Verify audit log row
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE event = 'auth.logout' AND metadata->>'session_id' = $1"
    )
    .bind(sid.into_inner().to_string())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert_eq!(audit_count, 1);
}

#[tokio::test]
async fn test_logout_idempotent_second_call_safe() {
    let harness = TestHarness::new().await;
    let (_uid, _sid, _ref_id, _fam, _raw, access_token) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;

    // First call: 200
    let req1 = Request::builder()
        .method("POST")
        .uri("/auth/logout")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();
    let resp1 = harness.router().oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);

    // Second call: idempotent, does not panic or corrupt data
    let req2 = Request::builder()
        .method("POST")
        .uri("/auth/logout")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();
    let resp2 = harness.router().oneshot(req2).await.unwrap();
    let (status, _) = response_to_json(resp2).await;
    assert!(status == StatusCode::OK || status == StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_concurrent_logout_wins_over_refresh() {
    let harness = TestHarness::new().await;
    let (_uid, sid, _ref_id, _fam, raw_token, access_token) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;

    let app1 = harness.router();
    let app2 = harness.router();

    let token_clone = raw_token.clone();
    let access_clone = access_token.clone();

    let h_logout = tokio::spawn(async move {
        let req = Request::builder()
            .method("POST")
            .uri("/auth/logout")
            .header(header::AUTHORIZATION, format!("Bearer {}", access_clone))
            .body(Body::empty())
            .unwrap();
        app1.oneshot(req).await.unwrap()
    });

    let h_refresh = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(15)).await;
        let req = Request::builder()
            .method("POST")
            .uri("/auth/refresh")
            .header(header::COOKIE, format!("__Secure-has_refresh={}", token_clone))
            .body(Body::empty())
            .unwrap();
        app2.oneshot(req).await.unwrap()
    });

    let (res_logout, res_refresh) = tokio::join!(h_logout, h_refresh);
    let resp_logout = res_logout.unwrap();
    let resp_refresh = res_refresh.unwrap();

    let (s_logout, _) = response_to_json(resp_logout).await;
    let (s_refresh, body_refresh) = response_to_json(resp_refresh).await;

    // Assertion 1: exactly one succeeds (logout OR refresh, never both)
    let exactly_one_succeeded = (s_logout == StatusCode::OK && s_refresh != StatusCode::OK)
        || (s_refresh == StatusCode::OK && s_logout != StatusCode::OK);
    assert!(exactly_one_succeeded, "Concurrent logout and refresh must never both succeed: logout={}, refresh={}", s_logout, s_refresh);

    // Assertion 2: if logout won, refresh returned 401 token_revoked
    if s_logout == StatusCode::OK {
        assert_eq!(s_refresh, StatusCode::UNAUTHORIZED);
        assert_eq!(body_refresh["error"]["code"], "token_revoked");
    }

    // Assertion 3: session is revoked regardless of which won
    let session_revoked_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        "SELECT revoked_at FROM sessions WHERE id = $1"
    )
    .bind(sid.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert!(session_revoked_at.is_some(), "Session must be revoked");

    // Assertion 4: no orphan refresh_tokens rows with revoked_at IS NULL
    let unrevoked_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM refresh_tokens WHERE session_id = $1 AND revoked_at IS NULL"
    )
    .bind(sid.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert_eq!(unrevoked_count, 0, "No orphan active refresh tokens must remain");
}

// ============================================================================
// 3. POST /auth/logout-all Tests
// ============================================================================

#[tokio::test]
async fn test_logout_all_happy_path() {
    let harness = TestHarness::new().await;
    let (uid, sid1, _ref_id, _fam, _raw, access_token1) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;
    let (sid2, _, _, _, _) = harness.add_session_for_user(uid, "device-2", "192.168.1.1").await;

    // Another user who must remain unaffected
    let (uid_other, sid_other, _, _, _, _) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;

    let req = Request::builder()
        .method("POST")
        .uri("/auth/logout-all")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token1))
        .body(Body::empty())
        .unwrap();

    let resp = harness.router().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(is_refresh_cookie_cleared(&resp));

    let (status, body) = response_to_json(resp).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["sessions_revoked"], 2);

    // Verify DB: user auth_version bumped to 2
    let auth_version: i64 = sqlx::query_scalar("SELECT auth_version FROM users WHERE id = $1")
        .bind(uid.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    assert_eq!(auth_version, 2);

    // Verify DB: all sessions of this user revoked
    let unrevoked_sessions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sessions WHERE user_id = $1 AND revoked_at IS NULL"
    )
    .bind(uid.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert_eq!(unrevoked_sessions, 0);

    // Verify DB: all refresh tokens of this user revoked
    let unrevoked_rt: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM refresh_tokens WHERE session_id IN ($1, $2) AND revoked_at IS NULL"
    )
    .bind(sid1.into_inner())
    .bind(sid2.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert_eq!(unrevoked_rt, 0);

    // Verify other user remains untouched
    let other_auth_version: i64 = sqlx::query_scalar("SELECT auth_version FROM users WHERE id = $1")
        .bind(uid_other.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    assert_eq!(other_auth_version, 1);

    let other_session_revoked: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT revoked_at FROM sessions WHERE id = $1")
            .bind(sid_other.into_inner())
            .fetch_one(&harness.state.db_pool)
            .await
            .unwrap();
    assert!(other_session_revoked.is_none());

    // Verify audit log
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE event = 'auth.logout_all' AND user_id = $1"
    )
    .bind(uid.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert_eq!(audit_count, 1);
}

#[tokio::test]
async fn test_logout_all_immediate_revocation_on_other_device() {
    let harness = TestHarness::new().await;
    let (uid, _sid1, _ref1, _fam1, _raw1, access_token1) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;
    let (_sid2, _ref2, _fam2, _raw2, access_token2) =
        harness.add_session_for_user(uid, "device-2", "192.168.1.2").await;

    // Device 2's access token hits /auth/me -> initially succeeds
    let req_pre = Request::builder()
        .method("GET")
        .uri("/auth/me")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token2))
        .body(Body::empty())
        .unwrap();
    let resp_pre = harness.router().oneshot(req_pre).await.unwrap();
    assert_eq!(resp_pre.status(), StatusCode::OK);

    // Device 1 calls /auth/logout-all
    let req_logout = Request::builder()
        .method("POST")
        .uri("/auth/logout-all")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token1))
        .body(Body::empty())
        .unwrap();
    let resp_logout = harness.router().oneshot(req_logout).await.unwrap();
    assert_eq!(resp_logout.status(), StatusCode::OK);

    // Device 2 IMMEDIATELY calls /auth/me with its token (without test clearing Redis cache)
    let req_post = Request::builder()
        .method("GET")
        .uri("/auth/me")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token2))
        .body(Body::empty())
        .unwrap();
    let resp_post = harness.router().oneshot(req_post).await.unwrap();
    let (status, body) = response_to_json(resp_post).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "token_revoked");
}

#[tokio::test]
async fn test_logout_all_during_in_flight_refresh() {
    let harness = TestHarness::new().await;
    let (uid, _sid_a, _ref_a, _fam_a, raw_token_a, _) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;
    let (_sid_b, _, _, _, access_token_b) =
        harness.add_session_for_user(uid, "device-b", "10.0.0.2").await;

    // 1. Device A performs refresh, acquiring new access token and new refresh token
    let req_refresh = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::COOKIE, format!("__Secure-has_refresh={}", raw_token_a))
        .body(Body::empty())
        .unwrap();

    let resp_refresh = harness.router().oneshot(req_refresh).await.unwrap();
    assert_eq!(resp_refresh.status(), StatusCode::OK);
    let new_refresh_cookie = extract_refresh_cookie_value(&resp_refresh).unwrap();
    let (_, body_refresh) = response_to_json(resp_refresh).await;
    let new_access_token_a = body_refresh["access_token"].as_str().unwrap().to_string();

    // 2. In-flight / immediate logout-all triggered from Device B
    let req_logout_all = Request::builder()
        .method("POST")
        .uri("/auth/logout-all")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token_b))
        .body(Body::empty())
        .unwrap();
    let resp_logout_all = harness.router().oneshot(req_logout_all).await.unwrap();
    assert_eq!(resp_logout_all.status(), StatusCode::OK);

    // Assertion 1: Device A's new access token is rejected by middleware
    let req_me = Request::builder()
        .method("GET")
        .uri("/auth/me")
        .header(header::AUTHORIZATION, format!("Bearer {}", new_access_token_a))
        .body(Body::empty())
        .unwrap();
    let resp_me = harness.router().oneshot(req_me).await.unwrap();
    let (status_me, body_me) = response_to_json(resp_me).await;
    assert_eq!(status_me, StatusCode::UNAUTHORIZED);
    assert_eq!(body_me["error"]["code"], "token_revoked");

    // Assertion 2: Device A's new refresh token is revoked (row exists with revoked_at IS NOT NULL)
    let new_raw_bytes = URL_SAFE_NO_PAD.decode(&new_refresh_cookie).unwrap();
    let new_token_hash = crypto::token::sha256_hex(&new_raw_bytes);
    let new_token_revoked_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        "SELECT revoked_at FROM refresh_tokens WHERE token_hash = $1"
    )
    .bind(&new_token_hash)
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert!(new_token_revoked_at.is_some(), "Device A's new refresh token must be marked revoked");
}

// ============================================================================
// 4. GET /auth/sessions Tests
// ============================================================================

#[tokio::test]
async fn test_sessions_list_happy_path() {
    let harness = TestHarness::new().await;
    let (uid, sid1, _ref1, _fam1, _raw1, access_token1) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;
    let (sid2, _, _, _, _) = harness.add_session_for_user(uid, "device-2", "2001:db8::1").await;

    let req = Request::builder()
        .method("GET")
        .uri("/auth/sessions")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token1))
        .body(Body::empty())
        .unwrap();

    let resp = harness.router().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let (status, body) = response_to_json(resp).await;
    assert_eq!(status, StatusCode::OK);

    let list = body.as_array().unwrap();
    assert_eq!(list.len(), 2);

    let current_session = list.iter().find(|s| s["id"] == sid1.into_inner().to_string()).unwrap();
    assert_eq!(current_session["current"], true);
    assert_eq!(current_session["ip_address"], "127.0.0.1");

    let other_session = list.iter().find(|s| s["id"] == sid2.into_inner().to_string()).unwrap();
    assert_eq!(other_session["current"], false);
    // IPv6 masked to /64
    assert!(other_session["ip_address"].as_str().unwrap().ends_with("/64"));
}

#[tokio::test]
async fn test_sessions_list_filters_revoked_and_expired() {
    let harness = TestHarness::new().await;
    let (uid, sid1, _ref1, _fam1, _raw1, access_token1) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;
    let (sid_revoked, _, _, _, _) = harness.add_session_for_user(uid, "revoked-device", "10.0.0.1").await;
    let (sid_expired, _, _, _, _) = harness.add_session_for_user(uid, "expired-device", "10.0.0.2").await;

    // Mark sid_revoked as revoked
    sqlx::query("UPDATE sessions SET revoked_at = now() WHERE id = $1")
        .bind(sid_revoked.into_inner())
        .execute(&harness.state.db_pool)
        .await
        .unwrap();

    // Mark sid_expired as expired
    sqlx::query("UPDATE sessions SET expires_at = now() - INTERVAL '1 hour' WHERE id = $1")
        .bind(sid_expired.into_inner())
        .execute(&harness.state.db_pool)
        .await
        .unwrap();

    let req = Request::builder()
        .method("GET")
        .uri("/auth/sessions")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token1))
        .body(Body::empty())
        .unwrap();

    let resp = harness.router().oneshot(req).await.unwrap();
    let (status, body) = response_to_json(resp).await;
    assert_eq!(status, StatusCode::OK);

    let list = body.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], sid1.into_inner().to_string());
}

#[tokio::test]
async fn test_sessions_list_caps_at_50() {
    let harness = TestHarness::new().await;
    let (uid, _sid1, _ref1, _fam1, _raw1, access_token1) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;

    // Insert 55 extra sessions
    let now = chrono::Utc::now();
    for i in 0..55 {
        let sid = SessionId::now_v7();
        sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, user_agent, ip_address, created_at, last_seen_at, expires_at, revoked_at)
            VALUES ($1, $2, $3, '127.0.0.1'::inet, $4, $4, $5, NULL)
            "#,
        )
        .bind(sid.into_inner())
        .bind(uid.into_inner())
        .bind(format!("batch-device-{}", i))
        .bind(now)
        .bind(now + chrono::Duration::days(30))
        .execute(&harness.state.db_pool)
        .await
        .unwrap();
    }

    let req = Request::builder()
        .method("GET")
        .uri("/auth/sessions")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token1))
        .body(Body::empty())
        .unwrap();

    let resp = harness.router().oneshot(req).await.unwrap();
    let (status, body) = response_to_json(resp).await;
    assert_eq!(status, StatusCode::OK);

    let list = body.as_array().unwrap();
    assert_eq!(list.len(), 50);
}

// ============================================================================
// 5. DELETE /auth/sessions/{id} Tests
// ============================================================================

#[tokio::test]
async fn test_session_delete_own_session_success() {
    let harness = TestHarness::new().await;
    let (uid, _sid1, _ref1, _fam1, _raw1, access_token1) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;
    let (sid2, _, _, _, _) = harness.add_session_for_user(uid, "device-2", "192.168.1.1").await;

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/auth/sessions/{}", sid2.into_inner()))
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token1))
        .body(Body::empty())
        .unwrap();

    let resp = harness.router().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    // Caller's cookie is NOT cleared because sid2 != sid1
    assert!(!is_refresh_cookie_cleared(&resp));

    let (status, body) = response_to_json(resp).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);

    // Verify DB: sid2 revoked
    let s2_revoked: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT revoked_at FROM sessions WHERE id = $1")
            .bind(sid2.into_inner())
            .fetch_one(&harness.state.db_pool)
            .await
            .unwrap();
    assert!(s2_revoked.is_some());

    // Verify DB: sid2 refresh tokens revoked
    let unrevoked_rt2: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM refresh_tokens WHERE session_id = $1 AND revoked_at IS NULL"
    )
    .bind(sid2.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert_eq!(unrevoked_rt2, 0);

    // Verify audit log
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE event = 'auth.session.revoked_remote' AND metadata->>'target_session_id' = $1"
    )
    .bind(sid2.into_inner().to_string())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert_eq!(audit_count, 1);
}

#[tokio::test]
async fn test_session_delete_current_session_clears_cookie() {
    let harness = TestHarness::new().await;
    let (_uid, sid1, _ref1, _fam1, _raw1, access_token1) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/auth/sessions/{}", sid1.into_inner()))
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token1))
        .body(Body::empty())
        .unwrap();

    let resp = harness.router().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(is_refresh_cookie_cleared(&resp));
}

#[tokio::test]
async fn test_session_delete_other_user_session_anti_enumeration_404() {
    let harness = TestHarness::new().await;
    let (_uid1, _sid1, _ref1, _fam1, _raw1, access_token1) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;
    let (_uid2, sid2, _ref2, _fam2, _raw2, _access_token2) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;

    // User 1 attempts to delete User 2's session
    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/auth/sessions/{}", sid2.into_inner()))
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token1))
        .body(Body::empty())
        .unwrap();

    let resp = harness.router().oneshot(req).await.unwrap();
    let (status, body) = response_to_json(resp).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");

    // Verify User 2's session was NOT revoked
    let s2_revoked: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT revoked_at FROM sessions WHERE id = $1")
            .bind(sid2.into_inner())
            .fetch_one(&harness.state.db_pool)
            .await
            .unwrap();
    assert!(s2_revoked.is_none());
}

#[tokio::test]
async fn test_session_delete_already_revoked_session_returns_404() {
    let harness = TestHarness::new().await;
    let (uid, _sid1, _ref1, _fam1, _raw1, access_token1) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;
    let (sid2, _, _, _, _) = harness.add_session_for_user(uid, "device-2", "192.168.1.1").await;

    // Revoke sid2 first
    sqlx::query("UPDATE sessions SET revoked_at = now() WHERE id = $1")
        .bind(sid2.into_inner())
        .execute(&harness.state.db_pool)
        .await
        .unwrap();

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/auth/sessions/{}", sid2.into_inner()))
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token1))
        .body(Body::empty())
        .unwrap();

    let resp = harness.router().oneshot(req).await.unwrap();
    let (status, body) = response_to_json(resp).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
}

#[tokio::test]
async fn test_session_delete_invalid_uuid_returns_400() {
    let harness = TestHarness::new().await;
    let (_uid, _sid1, _ref1, _fam1, _raw1, access_token1) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;

    let req = Request::builder()
        .method("DELETE")
        .uri("/auth/sessions/not-a-valid-uuid")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token1))
        .body(Body::empty())
        .unwrap();

    let resp = harness.router().oneshot(req).await.unwrap();
    let (status, body) = response_to_json(resp).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "validation_error");
}

// ============================================================================
// 6. Cross-cutting Tests (Audit, Error Shapes, Metrics)
// ============================================================================

#[tokio::test]
async fn test_cross_cutting_no_secrets_in_audit_log() {
    let harness = TestHarness::new().await;
    let (uid, _sid, _ref, _fam, raw_token, access_token) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;
    let (sid2, _, _, _, _) = harness.add_session_for_user(uid, "device-2", "192.168.1.1").await;

    // 1. Refresh
    let req_refresh = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::COOKIE, format!("__Secure-has_refresh={}", raw_token))
        .body(Body::empty())
        .unwrap();
    let _ = harness.router().oneshot(req_refresh).await.unwrap();

    // 2. Delete remote session
    let req_del = Request::builder()
        .method("DELETE")
        .uri(format!("/auth/sessions/{}", sid2.into_inner()))
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();
    let _ = harness.router().oneshot(req_del).await.unwrap();

    // 3. Logout
    let req_logout = Request::builder()
        .method("POST")
        .uri("/auth/logout")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();
    let _ = harness.router().oneshot(req_logout).await.unwrap();

    // Fetch all audit rows for this user
    let rows: Vec<(String, Option<Value>)> = sqlx::query_as(
        "SELECT event, metadata FROM audit_log WHERE user_id = $1"
    )
    .bind(uid.into_inner())
    .fetch_all(&harness.state.db_pool)
    .await
    .unwrap();

    assert!(!rows.is_empty());
    for (event, meta_opt) in rows {
        if let Some(meta) = meta_opt {
            let meta_str = meta.to_string();
            assert!(!meta_str.contains("secret"), "Audit row {} leaked secret: {}", event, meta_str);
            assert!(!meta_str.contains("password"), "Audit row {} leaked password: {}", event, meta_str);
            assert!(!meta_str.contains("token_hash"), "Audit row {} leaked token_hash: {}", event, meta_str);
            assert!(!meta_str.contains("otp"), "Audit row {} leaked otp: {}", event, meta_str);
            assert!(!meta_str.contains(&raw_token), "Audit row {} leaked raw_token: {}", event, meta_str);
        }
    }
}

#[tokio::test]
async fn test_cross_cutting_error_response_shape() {
    let harness = TestHarness::new().await;
    let req = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .body(Body::empty())
        .unwrap();

    let resp = harness.router().oneshot(req).await.unwrap();
    let (_, body) = response_to_json(resp).await;

    assert!(body.get("error").is_some());
    assert!(body["error"].get("code").is_some());
    assert!(body["error"].get("message").is_some());
}

#[tokio::test]
async fn test_cross_cutting_metrics_emitted() {
    let harness = TestHarness::new().await;
    let (_uid, _sid, _ref, _fam, raw_token, access_token) =
        harness.create_user_with_session(UserRole::Customer, UserStatus::Active, 1).await;

    // Trigger refresh
    let req1 = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::COOKIE, format!("__Secure-has_refresh={}", raw_token))
        .body(Body::empty())
        .unwrap();
    let _ = harness.router().oneshot(req1).await.unwrap();

    // Trigger logout
    let req2 = Request::builder()
        .method("POST")
        .uri("/auth/logout")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();
    let _ = harness.router().oneshot(req2).await.unwrap();

    // Check /metrics
    let req_metrics = Request::builder()
        .method("GET")
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();
    let resp_metrics = harness.router().oneshot(req_metrics).await.unwrap();
    let bytes = resp_metrics.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&bytes);

    assert!(body_str.contains("auth_token_refresh_total"));
    assert!(body_str.contains("auth_logout_total"));
}
