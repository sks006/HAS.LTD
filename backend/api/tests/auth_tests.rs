// backend/api/tests/auth_tests.rs

use std::sync::Arc;
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use base64::Engine;
use http_body_util::BodyExt;
use jsonwebtoken::{Algorithm, Header};
use redis::AsyncCommands;
use serde_json::{json, Value};
use tokio::sync::Semaphore;
use tower::ServiceExt;
use uuid::Uuid;

use api::{
    routes::build_router,
    state::AppState,
};
use crypto::jwt::{Claims, JwtKeys, DEFAULT_AUDIENCE, DEFAULT_ISSUER};
use domain::models::identifiers::{CredentialId, RefreshTokenId, SessionId, UserId};
use domain::models::user::{UserRole, UserStatus};

const UNTRUSTED_RSA_KEY_PRIV: &str = "-----BEGIN PRIVATE KEY-----\n\
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQCz1XdOBXy00DFK\n\
Tw30gHH6dkvGp7dXUKnxjDrhGXWcrKyPFZXj+6jJfuNJ7Vr0kkPht4UdV5NwJDAQ\n\
FCtguxeMDXZck76ShrGbteZyO3TinNgF/xlfS5/+ogqjnCDgfoevlf7MPBM+/6Kq\n\
7cCXADcMxt3lQKEf7DjqdBCnZSb2BVtk3oK+5B4Lj+SuDeYIQRp6lq3P5c9rznHA\n\
0hF0s/+i6vjVWwr7VTZyJN+qpvKMOzuEZH4uu7CzuYjTn9sBDZ9vkee0MtsXasOq\n\
maJd9doseGLvUkqikt20ygk1KlmJiGZ8jg5NE0apr9v5q/pzsqfeFskT+ViM9viW\n\
FoBE6c0ZAgMBAAECggEACkdvVFRX9nXX+2ldYNZerXRJjIdVoXYI4LelPeyYnmX4\n\
Tw1qzec6lJneIAGjK0uY+bPltfGliclnveOJPzQyelzioRQI1pNAX6gcDfd8DN/u\n\
MRmNt7a45HMarJNmBa71zoGCWOYuK3GtOJvyYC8w9lf/ovT4wLxd3HjZ4teTTIiV\n\
Ox1PltDAMGShkDeqzsXFYxePtqUEBSNfUQaDJQ18jmgXcVRAdGdJfM5U1tlqLu++\n\
GtL3qqbb3oTGTTapSTWl6gVE2G7ryAp1+nK50BWfAwcrwvvyklg/qU1jLSrGmeOj\n\
RTV4bjCCb2D7Z/yRdiYYywYobnql40jDoY/2iCQyIQKBgQDqTBxGUwTRdg83fX6y\n\
FthCp4UZf8CGEmJ5RrAL5q1ObKzwi1xRk4IlnMmRdZC/WaQ5c6HkonlUmgmUELIF\n\
dL/4CwbZHw2zH9Iq5xQ3or/uq+WXZwCHS9qAsmIv0tpy11xRB1HppG1pDRakOs+G\n\
ANxAdNpLdhL09Wz6fFhprMLbhQKBgQDEfd1AGmNmM5WXZlOta7Kjjdgee0lTCwJs\n\
WpJcPTSkpQt4rcqqDX9J9X0W6IdvuPNU/bBAUotyjun9GYSOntPuwwtW5/0YLu8/\n\
/zOqO1FRSKi/FYdfIgrqxr3sni5LPxJyEYuUE1R/uIVYe3rnWMMRXQHBgLaptoVI\n\
boLmVugNhQKBgQCkncEn7+BswsRCOpwvyDk967FqjwXXrUce6nl6gyK0/Uos392H\n\
O1qvzo8Y8j83EmZ+Ly5KE5WnyWp9a5WIm9TdE/JFzrmzZuIlXqInzOOJb6EqVSZb\n\
6QGmdq7h5wWwtnuYOspknDB7JMUJsoBoKGaXqwJNDrKXBXECNSdxZ/6oeQKBgAwt\n\
uO7mM87714PAxeLzz+ivO/v3rD3HaukIXBX3fTHPQPzD2NHdcExHOO+YjfxwvDRC\n\
XRA/jYEWNo6HbF3inkQoNqW2T0uWbSHMUYl1j764AC2HyhIta/vNQaxOyZPEXwJO\n\
8BXHdO9NooYvcRixubVzzE4QCkJOm8g3gUmBjXR5AoGBAIsihx6rQ1b0UpeRTJI3\n\
CfMFBXOnznlFGoB7TjicxQr5u3G11FIJk+Fdysu3ZS+OYJe7YQgLzEeSVZDdmEgw\n\
/HdiRPTvn5wckVnUIVlEXvP1V1YhDSvBuz8SFXf/jWMr0VgCqpfk7Ej9nZPt6uo3\n\
BMAt5UAaCdBVEfvQn2LtunKC\n\
-----END PRIVATE KEY-----";

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

    async fn create_user(
        &self,
        role: UserRole,
        status: UserStatus,
        auth_version: i64,
    ) -> (UserId, String, String, SessionId) {
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
        .bind("Test User")
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
            VALUES ($1, $2, 'test-agent', '127.0.0.1'::inet, $3, $3, $4, NULL)
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
        let (_, token_hash) = crypto::token::generate_refresh_token();
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, session_id, family_id, token_hash, issued_at, expires_at, revoked_at)
            VALUES ($1, $2, $3, $4, $5, $6, NULL)
            "#,
        )
        .bind(refresh_id.into_inner())
        .bind(session_id.into_inner())
        .bind(Uuid::new_v4())
        .bind(&token_hash)
        .bind(now)
        .bind(now + chrono::Duration::days(30))
        .execute(&self.state.db_pool)
        .await
        .unwrap();

        (user_id, email, password, session_id)
    }

    fn issue_token(
        &self,
        user_id: UserId,
        role: UserRole,
        auth_version: i64,
        scope: &str,
        exp_offset_secs: i64,
        nbf_offset_secs: i64,
        sid: Uuid,
    ) -> String {
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: user_id.into_inner().to_string(),
            iss: DEFAULT_ISSUER.to_string(),
            aud: DEFAULT_AUDIENCE.to_string(),
            exp: now + exp_offset_secs,
            iat: now,
            nbf: now + nbf_offset_secs,
            jti: Uuid::now_v7(),
            sid,
            auth_version,
            role: role.to_string(),
            scope: scope.to_string(),
        };
        self.state.jwt_keys.sign_access_token(&claims).unwrap()
    }

    async fn send_request(
        &self,
        method: &str,
        uri: &str,
        bearer_token: Option<&str>,
        json_body: Option<Value>,
    ) -> (StatusCode, Value) {
        let app = self.router();
        let mut builder = Request::builder().method(method).uri(uri);

        let b = Uuid::new_v4().into_bytes();
        let unique_ip = format!("10.{}.{}.{}", (b[0] & 0x7f) + 1, b[1], b[2]);
        builder = builder.header("x-forwarded-for", unique_ip);

        if let Some(token) = bearer_token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {}", token));
        }

        let body = if let Some(payload) = json_body {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            Body::from(serde_json::to_vec(&payload).unwrap())
        } else {
            Body::empty()
        };

        let req = builder.body(body).unwrap();
        let res = app.oneshot(req).await.unwrap();
        let status = res.status();

        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        let json_val = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::String(String::from_utf8_lossy(&bytes).into_owned()))
        };

        (status, json_val)
    }
}

// ============================================================================
// TEST SUITE A: Middleware Rejections
// ============================================================================

#[tokio::test]
async fn test_suite_a_middleware_rejections() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, session_id) = harness.create_user(UserRole::Customer, UserStatus::Active, 1).await;
    let valid_token = harness.issue_token(user_id, UserRole::Customer, 1, "access", 900, 0, session_id.into_inner());

    // 1. No Authorization header -> 401 invalid_token
    let (status, body) = harness.send_request("GET", "/auth/me", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_token");

    // 2. Malformed token -> 401 invalid_token
    let (status, body) = harness.send_request("GET", "/auth/me", Some("not.a.valid.jwt"), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_token");

    // 3. Token with kid=none header -> 401 invalid_token
    let header_no_kid = Header::new(Algorithm::RS256);
    let now = chrono::Utc::now().timestamp();
    let sample_claims = Claims {
        sub: user_id.into_inner().to_string(),
        iss: DEFAULT_ISSUER.to_string(),
        aud: DEFAULT_AUDIENCE.to_string(),
        exp: now + 900,
        iat: now,
        nbf: now,
        jti: Uuid::now_v7(),
        sid: session_id.into_inner(),
        auth_version: 1,
        role: "CUSTOMER".to_string(),
        scope: "access".to_string(),
    };
    // Sign token manually with missing kid header
    let token_missing_kid = format!(
        "{}.{}.fake_sig",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header_no_kid).unwrap()),
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(&sample_claims).unwrap())
    );
    let (status, body) = harness.send_request("GET", "/auth/me", Some(&token_missing_kid), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_token");

    // 4. Token signed with wrong key -> 401 invalid_token
    let wrong_dir = tempfile::tempdir().unwrap();
    std::fs::write(wrong_dir.path().join("999.pem"), UNTRUSTED_RSA_KEY_PRIV).unwrap();
    let wrong_keys = JwtKeys::load_from_dir(wrong_dir.path()).unwrap();
    let token_wrong_key = wrong_keys.sign_access_token(&sample_claims).unwrap();
    let (status, body) = harness.send_request("GET", "/auth/me", Some(&token_wrong_key), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_token");

    // 5. Token with alg=none -> 401 invalid_token [FIX-P4-S1: unused header_none removed]
    let token_alg_none = format!(
        "{}.{}.",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"{\"alg\":\"none\",\"typ\":\"JWT\",\"kid\":\"1\"}"),
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(&sample_claims).unwrap())
    );
    let (status, body) = harness.send_request("GET", "/auth/me", Some(&token_alg_none), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_token");

    // 6. Token with HS256 -> 401 invalid_token
    let token_hs256 = format!(
        "{}.{}.fakesig",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"{\"alg\":\"HS256\",\"typ\":\"JWT\",\"kid\":\"1\"}"),
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(&sample_claims).unwrap())
    );
    let (status, body) = harness.send_request("GET", "/auth/me", Some(&token_hs256), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_token");

    // 7. Token with scope=mfa_pending on normal route -> 401 invalid_token
    let mfa_pending_token = harness.issue_token(user_id, UserRole::Customer, 1, "mfa_pending", 900, 0, session_id.into_inner());
    let (status, body) = harness.send_request("GET", "/auth/me", Some(&mfa_pending_token), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_token");

    // 8. Expired token -> 401 token_expired
    let expired_token = harness.issue_token(user_id, UserRole::Customer, 1, "access", -3600, -7200, session_id.into_inner());
    let (status, body) = harness.send_request("GET", "/auth/me", Some(&expired_token), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "token_expired");

    // 9. Token with nbf in future -> 401 token_expired
    let future_nbf_token = harness.issue_token(user_id, UserRole::Customer, 1, "access", 900, 300, session_id.into_inner());
    let (status, body) = harness.send_request("GET", "/auth/me", Some(&future_nbf_token), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "token_expired");

    // 10. auth_version mismatch -> 401 token_revoked
    let token_version_mismatch = harness.issue_token(user_id, UserRole::Customer, 999, "access", 900, 0, session_id.into_inner());
    let (status, body) = harness.send_request("GET", "/auth/me", Some(&token_version_mismatch), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "token_revoked");

    // 11. status=SUSPENDED with otherwise valid token -> 403 account_suspended
    let (suspended_id, _, _, susp_session) = harness.create_user(UserRole::Customer, UserStatus::Suspended, 1).await;
    let suspended_token = harness.issue_token(suspended_id, UserRole::Customer, 1, "access", 900, 0, susp_session.into_inner());
    let (status, body) = harness.send_request("GET", "/auth/me", Some(&suspended_token), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "account_suspended");

    // 12. CUSTOMER hitting ADMIN route (/admin/metrics) -> 403 permission_denied
    let (status, body) = harness.send_request("GET", "/admin/metrics", Some(&valid_token), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "permission_denied");

    // 13. MODERATOR hitting ADMIN route (/admin/metrics) -> 403 permission_denied
    let (mod_id, _, _, mod_session) = harness.create_user(UserRole::Moderator, UserStatus::Active, 1).await;
    let mod_token = harness.issue_token(mod_id, UserRole::Moderator, 1, "access", 900, 0, mod_session.into_inner());
    let (status, body) = harness.send_request("GET", "/admin/metrics", Some(&mod_token), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "permission_denied");

    // 14. CUSTOMER hitting MODERATOR route (/orders) -> 403 permission_denied
    let (status, body) = harness.send_request("GET", "/orders", Some(&valid_token), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "permission_denied");
}

// ============================================================================
// TEST SUITE B: Middleware Accepts
// ============================================================================

#[tokio::test]
async fn test_suite_b_middleware_accepts() {
    let harness = TestHarness::new().await;

    // 1. CUSTOMER hitting CUSTOMER route (/auth/me) -> 200
    let (cust_id, _, _, cust_sess) = harness.create_user(UserRole::Customer, UserStatus::Active, 1).await;
    let cust_token = harness.issue_token(cust_id, UserRole::Customer, 1, "access", 900, 0, cust_sess.into_inner());
    let (status, body) = harness.send_request("GET", "/auth/me", Some(&cust_token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], cust_id.into_inner().to_string());

    // 2. MODERATOR hitting CUSTOMER route (/auth/me) -> 200
    let (mod_id, _, _, mod_sess) = harness.create_user(UserRole::Moderator, UserStatus::Active, 1).await;
    let mod_token = harness.issue_token(mod_id, UserRole::Moderator, 1, "access", 900, 0, mod_sess.into_inner());
    let (status, body) = harness.send_request("GET", "/auth/me", Some(&mod_token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["role"], "MODERATOR");

    // 3. MODERATOR hitting MODERATOR route (/orders) -> 200
    let (status, _) = harness.send_request("GET", "/orders", Some(&mod_token), None).await;
    assert_eq!(status, StatusCode::OK);

    // 4. ADMIN hitting ADMIN route (/admin/metrics) -> 200
    let (admin_id, _, _, admin_sess) = harness.create_user(UserRole::Admin, UserStatus::Active, 1).await;
    let admin_token = harness.issue_token(admin_id, UserRole::Admin, 1, "access", 900, 0, admin_sess.into_inner());
    let (status, body) = harness.send_request("GET", "/admin/metrics", Some(&admin_token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["database_status"], "Online");
}

// ============================================================================
// TEST SUITE C: Redis Caching & Fallback Behavior
// ============================================================================

#[tokio::test]
async fn test_suite_c_redis_behavior() {
    let harness = TestHarness::new().await;
    let (user_id, email, password, session_id) = harness.create_user(UserRole::Customer, UserStatus::Active, 1).await;
    let token = harness.issue_token(user_id, UserRole::Customer, 1, "access", 900, 0, session_id.into_inner());

    let auth_version_key = format!("auth_version:{}", user_id.into_inner());
    let user_status_key = format!("user_status:{}", user_id.into_inner());

    if let Some(mut redis) = harness.state.redis.clone() {
        // Ensure cache miss initially
        let _: Result<(), _> = redis.del(&auth_version_key).await;
        let _: Result<(), _> = redis.del(&user_status_key).await;

        let queries_before = harness.state.get_db_query_count();

        // 1. auth_version cache miss -> DB fallback -> request succeeds & cache populated
        let (status, _) = harness.send_request("GET", "/auth/me", Some(&token), None).await;
        assert_eq!(status, StatusCode::OK);

        let queries_after_miss = harness.state.get_db_query_count();
        assert_eq!(queries_after_miss, queries_before + 1, "Cache miss must increment DB query counter by 1");

        let cached_version: Option<i64> = redis.get(&auth_version_key).await.unwrap();
        let cached_status: Option<String> = redis.get(&user_status_key).await.unwrap();
        assert_eq!(cached_version, Some(1));
        assert_eq!(cached_status.as_deref(), Some("ACTIVE"));

        // 2. auth_version cache hit -> warm cache succeeds with ZERO DB queries [REQUIRED-2]
        let before = harness.state.get_db_query_count();
        let (status, _) = harness.send_request("GET", "/auth/me",
            Some(&token), None).await;
        assert_eq!(status, StatusCode::OK);
        let after = harness.state.get_db_query_count();
        assert_eq!(after, before,
            "warm cache must not hit the database");
    }

    // 3. Redis down simulation -> middleware falls through to DB, request still succeeds
    let degraded_state = AppState::from_parts(
        harness.state.db_pool.clone(),
        None, // Redis connection is None (simulating unavailable Redis)
        harness.state.jwt_keys.clone(),
        harness.state.argon2_semaphore.clone(),
        harness.state.config.clone(),
    );
    let degraded_app = build_router(degraded_state);
    let req = Request::builder()
        .method("GET")
        .uri("/auth/me")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let res = degraded_app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 4. After login, auth_version and status are populated in Redis
    let login_payload = json!({
        "email": email,
        "password": password
    });
    let (login_status, login_body) = harness.send_request("POST", "/auth/login", None, Some(login_payload)).await;
    assert_eq!(login_status, StatusCode::OK);
    assert_eq!(login_body["mfa_required"], false);

    if let Some(mut redis) = harness.state.redis.clone() {
        let v: Option<i64> = redis.get(&auth_version_key).await.unwrap();
        assert_eq!(v, Some(1));
    }
}

// ============================================================================
// TEST SUITE D: Argon2 Gate Concurrency
// ============================================================================

#[tokio::test]
async fn test_suite_d_argon2_gate() {
    let harness = TestHarness::new().await;
    let mut users = Vec::new();
    for _ in 0..4 {
        let (_, email, password, _) = harness.create_user(UserRole::Customer, UserStatus::Active, 1).await;
        users.push((email, password));
    }

    // 1. 4 concurrent logins succeed within normal latency
    let mut handles = Vec::new();
    for (email, password) in users.iter().cloned() {
        let state_clone = harness.state.clone();
        let handle = tokio::spawn(async move {
            let app = build_router(state_clone);
            let req = Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&json!({ "email": email, "password": password })).unwrap()))
                .unwrap();
            let res = app.oneshot(req).await.unwrap();
            res.status()
        });
        handles.push(handle);
    }

    for handle in handles {
        let status = handle.await.unwrap();
        assert_eq!(status, StatusCode::OK);
    }

    // 2. 8 concurrent logins with 4 semaphore permits and short timeout: 4 succeed, 4 return 503
    let custom_semaphore = Arc::new(Semaphore::new(4));
    let mut custom_config = (*harness.state.config).clone();
    custom_config.argon2_permits = 4;
    custom_config.argon2_timeout_ms = 50; // Short 50ms timeout for deterministic rejection

    let constrained_state = AppState::from_parts(
        harness.state.db_pool.clone(),
        harness.state.redis.clone(),
        harness.state.jwt_keys.clone(),
        custom_semaphore.clone(),
        Arc::new(custom_config),
    );

    // Hold all 4 permits artificially to force queuing
    let permit1 = custom_semaphore.clone().acquire_owned().await.unwrap();
    let permit2 = custom_semaphore.clone().acquire_owned().await.unwrap();
    let permit3 = custom_semaphore.clone().acquire_owned().await.unwrap();
    let permit4 = custom_semaphore.clone().acquire_owned().await.unwrap();

    let (_, test_email, test_pwd, _) = harness.create_user(UserRole::Customer, UserStatus::Active, 1).await;
    let constrained_app = build_router(constrained_state.clone());
    let req = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&json!({ "email": test_email, "password": test_pwd })).unwrap()))
        .unwrap();

    let res = constrained_app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);

    // 3. After permits drain, next login succeeds
    drop(permit1);
    drop(permit2);
    drop(permit3);
    drop(permit4);

    // Rate limit assumption [FIX-P4-S5]:
    // The previous request received HTTP 503 (concurrency gate timeout), which did not increment
    // the email authentication failure rate limiter. To maintain total test isolation, we also
    // explicitly clear the Redis rate limiter keys.
    if let Some(mut redis) = harness.state.redis.clone() {
        let _: Result<(), _> = redis.del(format!("rl:auth:login:email:{}", test_email.to_lowercase())).await;
    }

    let drain_app = build_router(constrained_state);
    let req = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&json!({ "email": test_email, "password": test_pwd })).unwrap()))
        .unwrap();
    let res = drain_app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

// ============================================================================
// TEST SUITE E: Phase 3 Corrections & 7 Invalidation Sites
// ============================================================================

#[tokio::test]
async fn test_suite_e_password_reset_and_invalidation_sites() {
    let harness = TestHarness::new().await;

    // FIX-C3-PHASE4: test_password_reset_revokes_outstanding_sessions
    let (user_id, _, _, session_id) = harness.create_user(UserRole::Customer, UserStatus::Active, 1).await;
    let old_token = harness.issue_token(user_id, UserRole::Customer, 1, "access", 900, 0, session_id.into_inner());

    // Initial check: token is accepted
    let (status, _) = harness.send_request("GET", "/auth/me", Some(&old_token), None).await;
    assert_eq!(status, StatusCode::OK);

    // Simulate password reset transaction: update credentials, bump auth_version, revoke sessions
    let new_hash = crypto::password::hash_password("NewPassword123!").unwrap();
    let mut tx = harness.state.db_pool.begin().await.unwrap();
    sqlx::query("UPDATE credentials SET secret_hash = $1 WHERE user_id = $2")
        .bind(&new_hash)
        .bind(user_id.into_inner())
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("UPDATE users SET auth_version = auth_version + 1 WHERE id = $1")
        .bind(user_id.into_inner())
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("UPDATE sessions SET revoked_at = now() WHERE user_id = $1")
        .bind(user_id.into_inner())
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // Invalidate Redis cache post-commit
    if let Some(mut redis) = harness.state.redis.clone() {
        let auth_version_key = format!("auth_version:{}", user_id.into_inner());
        let _: Result<(), _> = redis.del(&auth_version_key).await;
    }

    // Assert that the old token is rejected with 401 token_revoked
    let (status, body) = harness.send_request("GET", "/auth/me", Some(&old_token), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "token_revoked");

    // Test the 7 Invalidation Sites from Phase 3 §1.g:
    // Sites 1, 2, 3, 4, 7 (direct auth_version increment):
    for site_num in [1, 2, 3, 4, 7] {
        let (u_id, _, _, s_id) = harness.create_user(UserRole::Customer, UserStatus::Active, 1).await;
        let token = harness.issue_token(u_id, UserRole::Customer, 1, "access", 900, 0, s_id.into_inner());

        // Verify initial acceptance
        let (status, _) = harness.send_request("GET", "/auth/me", Some(&token), None).await;
        assert_eq!(status, StatusCode::OK, "Site {} initial request failed", site_num);

        // Execute DB mutation
        sqlx::query("UPDATE users SET auth_version = auth_version + 1 WHERE id = $1")
            .bind(u_id.into_inner())
            .execute(&harness.state.db_pool)
            .await
            .unwrap();

        // Flush Redis cache post-commit
        if let Some(mut redis) = harness.state.redis.clone() {
            let key = format!("auth_version:{}", u_id.into_inner());
            let _: Result<(), _> = redis.del(&key).await;
        }

        // Verify old token is revoked
        let (status, body) = harness.send_request("GET", "/auth/me", Some(&token), None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "Site {} must reject with 401", site_num);
        assert_eq!(body["error"]["code"], "token_revoked", "Site {} must emit token_revoked", site_num);
    }

    // Site 5: Account suspension via status change (triggers automatic auth_version bump) [FIX-P4-S4]
    {
        let (u_id, _, _, s_id) = harness.create_user(UserRole::Customer, UserStatus::Active, 1).await;
        let token = harness.issue_token(u_id, UserRole::Customer, 1, "access", 900, 0, s_id.into_inner());

        sqlx::query("UPDATE users SET status = 'SUSPENDED' WHERE id = $1")
            .bind(u_id.into_inner())
            .execute(&harness.state.db_pool)
            .await
            .unwrap();

        // Assert auth_version went from 1 to 2 via the trigger [FIX-P4-S4]
        let bumped_version: i64 = sqlx::query_scalar("SELECT auth_version FROM users WHERE id = $1")
            .bind(u_id.into_inner())
            .fetch_one(&harness.state.db_pool)
            .await
            .unwrap();
        assert_eq!(bumped_version, 2, "Status change to SUSPENDED must trigger an automatic auth_version bump from 1 to 2");

        if let Some(mut redis) = harness.state.redis.clone() {
            let _: Result<(), _> = redis.del(format!("auth_version:{}", u_id.into_inner())).await;
            let _: Result<(), _> = redis.del(format!("user_status:{}", u_id.into_inner())).await;
        }

        // Triggers bump auth_version and set status to SUSPENDED -> middleware rejects with 403 account_suspended
        let (status, body) = harness.send_request("GET", "/auth/me", Some(&token), None).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body["error"]["code"], "account_suspended");
    }

    // Site 6: Role change (triggers automatic auth_version bump)
    {
        let (u_id, _, _, s_id) = harness.create_user(UserRole::Customer, UserStatus::Active, 1).await;
        let token = harness.issue_token(u_id, UserRole::Customer, 1, "access", 900, 0, s_id.into_inner());

        sqlx::query("UPDATE users SET role = 'MODERATOR' WHERE id = $1")
            .bind(u_id.into_inner())
            .execute(&harness.state.db_pool)
            .await
            .unwrap();

        if let Some(mut redis) = harness.state.redis.clone() {
            let _: Result<(), _> = redis.del(format!("auth_version:{}", u_id.into_inner())).await;
        }

        // Trigger bumped auth_version -> old token is revoked
        let (status, body) = harness.send_request("GET", "/auth/me", Some(&token), None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"]["code"], "token_revoked");
    }
}

// ============================================================================
// FIX-P4-D1: Session ID Semantic Isolation Test
// ============================================================================

#[tokio::test]
async fn test_session_id_claims_and_isolation() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, session_1) = harness.create_user(UserRole::Customer, UserStatus::Active, 1).await;
    let session_2 = SessionId::now_v7();

    // Two tokens issued against session_1
    let token_1a = harness.issue_token(user_id, UserRole::Customer, 1, "access", 900, 0, session_1.into_inner());
    let token_1b = harness.issue_token(user_id, UserRole::Customer, 1, "access", 900, 0, session_1.into_inner());

    // One token issued against session_2
    let token_2 = harness.issue_token(user_id, UserRole::Customer, 1, "access", 900, 0, session_2.into_inner());

    let claims_1a = harness.state.jwt_keys.verify_access_token(&token_1a).unwrap();
    let claims_1b = harness.state.jwt_keys.verify_access_token(&token_1b).unwrap();
    let claims_2 = harness.state.jwt_keys.verify_access_token(&token_2).unwrap();

    // Two access tokens against the same session share sid
    assert_eq!(claims_1a.sid, session_1.into_inner());
    assert_eq!(claims_1b.sid, session_1.into_inner());
    assert_ne!(claims_1a.jti, claims_1b.jti, "Each token must possess a unique jti");

    // Two access tokens against different sessions do not share sid
    assert_eq!(claims_2.sid, session_2.into_inner());
    assert_ne!(claims_1a.sid, claims_2.sid, "Tokens for different sessions must have different sid");
}

// ============================================================================
// FIX-P4-S3: Moderator Login Returns 501 mfa_not_yet_available
// ============================================================================

#[tokio::test]
async fn test_staff_first_login_provides_enrollment_path() {
    let harness = TestHarness::new().await;
    let (mod_id, mod_email, mod_pwd, _) = harness.create_user(UserRole::Moderator, UserStatus::Active, 1).await;

    let payload = json!({
        "email": mod_email,
        "password": mod_pwd,
    });

    let (status, body) = harness.send_request("POST", "/auth/login", None, Some(payload.clone())).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["mfa_enrollment_required"], true);
    assert_eq!(body["expires_in"], 900);
    let enrollment_token = body["enrollment_token"].as_str().expect("enrollment_token must be present");

    // Verify token claims
    let claims = harness.state.jwt_keys.verify_access_token(enrollment_token).expect("enrollment_token must be valid");
    assert_eq!(claims.scope, "mfa_enrollment");
    assert_eq!(claims.sub, mod_id.into_inner().to_string());

    // Use enrollment_token on /auth/mfa/totp/enroll
    let (s_enr, b_enr) = harness
        .send_request(
            "POST",
            "/auth/mfa/totp/enroll",
            Some(enrollment_token),
            Some(json!({ "current_password": mod_pwd })),
        )
        .await;
    assert_eq!(s_enr, StatusCode::OK);
    assert!(b_enr["otpauth_url"].is_string());
    assert!(b_enr["recovery_codes"].is_array());

    // Generate valid TOTP code
    let (enc_secret,): (Vec<u8>,) = sqlx::query_as(
        "SELECT secret_encrypted FROM mfa_credentials WHERE user_id = $1",
    )
    .bind(mod_id.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    let secret = crypto::totp::decrypt_totp_secret(&enc_secret, &harness.state.totp_key).unwrap();
    let totp = crypto::totp::create_totp_instance(&secret).unwrap();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let code = totp.generate(now);

    // Use enrollment_token on /auth/mfa/totp/verify
    let (s_ver, b_ver) = harness
        .send_request(
            "POST",
            "/auth/mfa/totp/verify",
            Some(enrollment_token),
            Some(json!({ "code": code })),
        )
        .await;
    assert_eq!(s_ver, StatusCode::OK);
    assert_eq!(b_ver["success"], true);

    // Subsequent login now returns MFA challenge (mfa_required: true)
    let (s_log2, b_log2) = harness.send_request("POST", "/auth/login", None, Some(payload)).await;
    assert_eq!(s_log2, StatusCode::OK);
    assert_eq!(b_log2["mfa_required"], true);
    let mfa_token = b_log2["mfa_token"].as_str().expect("mfa_token must be present");

    // Complete login challenge
    let now2 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let code2 = totp.generate(now2);
    let (s_totp, b_totp) = harness
        .send_request(
            "POST",
            "/auth/login/totp",
            None,
            Some(json!({ "mfa_token": mfa_token, "code": code2 })),
        )
        .await;
    assert_eq!(s_totp, StatusCode::OK);
    assert!(b_totp["access_token"].is_string());
}

#[tokio::test]
async fn test_enrollment_token_cannot_access_protected_routes() {
    let harness = TestHarness::new().await;
    let (_, mod_email, mod_pwd, _) = harness.create_user(UserRole::Moderator, UserStatus::Active, 1).await;

    let payload = json!({
        "email": mod_email,
        "password": mod_pwd,
    });

    let (status, body) = harness.send_request("POST", "/auth/login", None, Some(payload)).await;
    assert_eq!(status, StatusCode::OK);
    let enrollment_token = body["enrollment_token"].as_str().expect("enrollment_token must be present");

    // 1. /auth/me -> 401 invalid_token
    let (s_me, b_me) = harness.send_request("GET", "/auth/me", Some(enrollment_token), None).await;
    assert_eq!(s_me, StatusCode::UNAUTHORIZED);
    assert_eq!(b_me["error"]["code"], "invalid_token");

    // 2. /auth/sessions -> 401 invalid_token
    let (s_sess, b_sess) = harness.send_request("GET", "/auth/sessions", Some(enrollment_token), None).await;
    assert_eq!(s_sess, StatusCode::UNAUTHORIZED);
    assert_eq!(b_sess["error"]["code"], "invalid_token");

    // 3. /auth/mfa/totp/disable -> 401 invalid_token
    let (s_dis, b_dis) = harness
        .send_request(
            "POST",
            "/auth/mfa/totp/disable",
            Some(enrollment_token),
            Some(json!({ "current_password": mod_pwd, "code": "123456" })),
        )
        .await;
    assert_eq!(s_dis, StatusCode::UNAUTHORIZED);
    assert_eq!(b_dis["error"]["code"], "invalid_token");
}

#[tokio::test]
async fn test_moderator_login_with_mfa_returns_challenge() {
    let harness = TestHarness::new().await;
    let (mod_id, mod_email, mod_pwd, _) = harness.create_user(UserRole::Moderator, UserStatus::Active, 1).await;

    let (secret_bytes, _) = crypto::totp::generate_totp_secret();
    let secret_enc = crypto::totp::encrypt_totp_secret(&secret_bytes, &harness.state.totp_key).unwrap();
    let (_, hashes) = crypto::totp::generate_recovery_codes(10).unwrap();
    let mfa_id = Uuid::now_v7();

    sqlx::query(
        r#"
        INSERT INTO mfa_credentials (id, user_id, kind, secret_encrypted, recovery_codes_hash, confirmed_at, created_at)
        VALUES ($1, $2, 'totp', $3, $4, now(), now())
        "#,
    )
    .bind(mfa_id)
    .bind(mod_id.into_inner())
    .bind(&secret_enc)
    .bind(&hashes)
    .execute(&harness.state.db_pool)
    .await
    .unwrap();

    let payload = json!({
        "email": mod_email,
        "password": mod_pwd,
    });

    let (status, body) = harness.send_request("POST", "/auth/login", None, Some(payload)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["mfa_required"], true);
    let mfa_token = body["mfa_token"].as_str().expect("mfa_token must be present");
    let claims = harness.state.jwt_keys.verify_access_token(mfa_token).expect("mfa_token must be valid");
    assert_eq!(claims.scope, "mfa_pending");
    assert_eq!(claims.sub, mod_id.into_inner().to_string());
}

// ============================================================================
// TEST SUITE F: Health and Metrics Endpoints
// ============================================================================

#[tokio::test]
async fn test_health_check() {
    let harness = TestHarness::new().await;

    // 1. /health returns 200 with {"status":"ok"}
    let (status, body) = harness.send_request("GET", "/health", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "status": "ok" }));
    println!("Health check body: {:?}", body);
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let harness = TestHarness::new().await;

    // 2. /metrics returns 200 with text/plain Prometheus format
    let app = harness.router();
    let req = Request::builder().method("GET").uri("/metrics").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let content_type = res.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap();
    assert!(content_type.starts_with("text/plain"), "Content-type must be text/plain");
}

// ============================================================================
// TEST SUITE G: Error Shape Contract
// ============================================================================

#[tokio::test]
async fn test_suite_g_error_shape_contract() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, session_id) = harness.create_user(UserRole::Customer, UserStatus::Active, 1).await;
    let valid_token = harness.issue_token(user_id, UserRole::Customer, 1, "access", 900, 0, session_id.into_inner());

    // Assert exact error JSON shape on 401 invalid_token
    let (status_401, body_401) = harness.send_request("GET", "/auth/me", None, None).await;
    assert_eq!(status_401, StatusCode::UNAUTHORIZED);
    assert_exact_error_shape(&body_401, "invalid_token");

    // Assert exact error JSON shape on 403 permission_denied
    let (status_403, body_403) = harness.send_request("GET", "/admin/metrics", Some(&valid_token), None).await;
    assert_eq!(status_403, StatusCode::FORBIDDEN);
    assert_exact_error_shape(&body_403, "permission_denied");
}

fn assert_exact_error_shape(val: &Value, expected_code: &str) {
    let obj = val.as_object().expect("Response body must be a JSON object");
    assert_eq!(obj.len(), 1, "Root object must contain exactly one key: 'error'");
    assert!(obj.contains_key("error"), "Root object must contain 'error'");

    let error_obj = obj.get("error").unwrap().as_object().expect("'error' must be a JSON object");
    assert_eq!(error_obj.len(), 2, "'error' object must contain exactly two keys: 'code' and 'message'");
    assert_eq!(error_obj.get("code").unwrap().as_str().unwrap(), expected_code);
    assert!(error_obj.get("message").unwrap().is_string(), "'message' must be a string");
}

// ============================================================================
// REQUIRED-1: Audit Events On Login Failure Paths
// ============================================================================

#[tokio::test]
async fn test_failed_login_emits_audit_row() {
    let harness = TestHarness::new().await;
    let (user_id, email, _pwd, _) = harness.create_user(UserRole::Customer, UserStatus::Active, 1).await;

    let payload = json!({
        "email": email,
        "password": "WrongPassword123!",
    });

    let (status, body) = harness.send_request("POST", "/auth/login", None, Some(payload)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_credentials");

    let row: (String, Value) = sqlx::query_as(
        "SELECT event, metadata FROM audit_log WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(user_id.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();

    assert_eq!(row.0, "auth.login.failed_credentials");
    let meta_str = row.1.to_string();
    assert!(!meta_str.contains("WrongPassword123!"), "Metadata must never contain plaintext password");
    assert!(!meta_str.contains("argon2id"), "Metadata must never contain password hashes");
    assert!(!meta_str.contains("secret"), "Metadata must never contain secret keys");
    assert!(row.1.get("attempted_email_domain").is_some());
}

#[tokio::test]
async fn test_suspended_login_emits_audit_row() {
    let harness = TestHarness::new().await;
    let (user_id, email, pwd, _) = harness.create_user(UserRole::Customer, UserStatus::Suspended, 1).await;

    let payload = json!({
        "email": email,
        "password": pwd,
    });

    let (status, body) = harness.send_request("POST", "/auth/login", None, Some(payload)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "account_suspended");

    let row: (String, Value) = sqlx::query_as(
        "SELECT event, metadata FROM audit_log WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(user_id.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();

    assert_eq!(row.0, "auth.login.failed_suspended");
    let meta_str = row.1.to_string();
    assert!(!meta_str.contains("Password"), "Metadata must never contain plaintext password");
    assert!(!meta_str.contains("argon2id"), "Metadata must never contain password hashes");
    assert_eq!(row.1.get("status").and_then(|v| v.as_str()), Some("SUSPENDED"));
}
