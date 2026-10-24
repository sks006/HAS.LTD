// backend/api/tests/email_flow_tests.rs
//! Phase 6A: Email Flows Integration Tests
//! Covers:
//! - POST /auth/register
//! - POST /auth/register/verify-otp
//! - POST /auth/register/resend-otp
//! - POST /auth/password/forgot
//! - POST /auth/password/reset
//! - POST /auth/password/change
//! - Cross-cutting audit logs, rate limits, and error shapes.

use std::time::Instant;
use axum::{
    body::Body,
    http::{header, HeaderMap, Request, StatusCode},
};
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
use domain::ports::email_port::EmailMessage;

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

    async fn send_request(
        &self,
        method: &str,
        uri: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let (status, val, _) = self.send_request_with_headers(method, uri, token, body, &[]).await;
        (status, val)
    }

    async fn send_request_with_headers(
        &self,
        method: &str,
        uri: &str,
        token: Option<&str>,
        body: Option<Value>,
        extra_headers: &[(&str, &str)],
    ) -> (StatusCode, Value, HeaderMap) {
        let mut req_builder = Request::builder()
            .method(method)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json");

        if let Some(t) = token {
            req_builder = req_builder.header(header::AUTHORIZATION, format!("Bearer {}", t));
        }

        let mut has_xff = false;
        for (k, v) in extra_headers {
            if k.eq_ignore_ascii_case("x-forwarded-for") {
                has_xff = true;
            }
            req_builder = req_builder.header(*k, *v);
        }

        if !has_xff {
            let b = Uuid::new_v4().into_bytes();
            let unique_ip = format!("10.{}.{}.{}", (b[0] & 0x7f) + 1, b[1], b[2]);
            req_builder = req_builder.header("x-forwarded-for", unique_ip);
        }

        let body_bytes = match body {
            Some(v) => serde_json::to_vec(&v).unwrap(),
            None => Vec::new(),
        };

        let req = req_builder.body(Body::from(body_bytes)).unwrap();
        let resp = self.router().oneshot(req).await.unwrap();
        let headers = resp.headers().clone();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
        (status, json, headers)
    }

    #[allow(dead_code)]
    fn captured_emails(&self) -> Vec<EmailMessage> {
        self.state.in_memory_email.as_ref().unwrap().all()
    }

    async fn drain_emails(&self) -> Vec<EmailMessage> {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        self.state.in_memory_email.as_ref().unwrap().drain()
    }

    async fn create_user_with_status(&self, status: UserStatus) -> String {
        let user_id = UserId::now_v7();
        let email = format!("user_{}_{}@has.ltd", status.to_string().to_lowercase(), Uuid::now_v7());
        let now = chrono::Utc::now();

        sqlx::query(
            r#"
            INSERT INTO users (id, email, email_normalized, name, role, status, auth_version, email_verified_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, 'CUSTOMER', $5, 1, $6, $6, $6)
            "#,
        )
        .bind(user_id.into_inner())
        .bind(&email)
        .bind(&email)
        .bind("Test User")
        .bind(status.to_string())
        .bind(now)
        .execute(&self.state.db_pool)
        .await
        .unwrap();

        email
    }

    fn clear_emails(&self) {
        self.state.in_memory_email.as_ref().unwrap().clear();
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

    async fn create_user_with_session(
        &self,
        role: UserRole,
        status: UserStatus,
        auth_version: i64,
    ) -> (UserId, SessionId, RefreshTokenId, Uuid, String, String, String) {
        let user_id = UserId::now_v7();
        let email = format!("test_{}@has.ltd", Uuid::now_v7());
        let password = "Password123!".to_string();
        let password_hash = crypto::password::hash_password(&password).unwrap();
        let now = chrono::Utc::now();

        sqlx::query(
            r#"
            INSERT INTO users (id, email, email_normalized, name, role, status, auth_version, email_verified_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8, $8)
            "#,
        )
        .bind(user_id.into_inner())
        .bind(&email)
        .bind(&email)
        .bind("Test User")
        .bind(role.as_str())
        .bind(status.as_str())
        .bind(auth_version)
        .bind(if status == UserStatus::Active { Some(now) } else { None })
        .execute(&self.state.db_pool)
        .await
        .unwrap();

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

        let access_token = self.issue_token(user_id, role, auth_version, "access", session_id);

        (user_id, session_id, refresh_id, family_id, raw_token, access_token, email)
    }

    async fn add_extra_session(
        &self,
        user_id: UserId,
    ) -> (SessionId, RefreshTokenId, String, String) {
        let session_id = SessionId::now_v7();
        let now = chrono::Utc::now();
        sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, user_agent, ip_address, created_at, last_seen_at, expires_at, revoked_at)
            VALUES ($1, $2, 'extra-agent/1.0', '127.0.0.2'::inet, $3, $3, $4, NULL)
            "#,
        )
        .bind(session_id.into_inner())
        .bind(user_id.into_inner())
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
        (session_id, refresh_id, raw_token, access_token)
    }
}

// ============================================================================
// 1. REGISTRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_register_new_email_creates_user_and_credentials_and_redis_otp() {
    let harness = TestHarness::new().await;
    harness.clear_emails();

    let email = format!("reg_new_{}@has.ltd", Uuid::now_v7());
    let payload = json!({
        "email": email,
        "password": "Password123!",
        "name": "New User"
    });

    let (status, body) = harness.send_request("POST", "/auth/register", None, Some(payload)).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body["status"], "pending_verification");
    assert_eq!(body["message"], "If eligible, a verification code has been dispatched.");

    // DB assertions
    let row: (Uuid, String, Option<chrono::DateTime<chrono::Utc>>) = sqlx::query_as(
        "SELECT id, status, email_verified_at FROM users WHERE email = $1",
    )
    .bind(&email)
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();

    let (user_id, user_status, email_verified_at) = row;
    assert_eq!(user_status, "PENDING");
    assert!(email_verified_at.is_none());

    // Credentials assertions
    let cred_hash: (String,) = sqlx::query_as(
        "SELECT secret_hash FROM credentials WHERE user_id = $1 AND kind = 'password'",
    )
    .bind(user_id)
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert!(cred_hash.0.starts_with("$argon2id$"));

    // Redis OTP assertion
    let mut redis = harness.state.redis.clone().unwrap();
    let otp_key = format!("otp:email_verify:{}", user_id);
    let hash: String = redis.get(&otp_key).await.unwrap();
    assert_eq!(hash.len(), 64);
    let ttl: i64 = redis.ttl(&otp_key).await.unwrap();
    assert!(ttl <= 600 && ttl > 550);

    // Email capture assertion
    let emails = harness.drain_emails().await;
    let captured = emails.into_iter().find(|m| m.to == email).expect("Email must be captured");
    assert_eq!(captured.template_id, "email_verify_otp");
    let code = captured.variables["code"].as_str().unwrap();
    assert_eq!(code.len(), 6);
    assert!(crypto::otp::verify_otp_hash(&harness.state.email_pepper, user_id, code, &hash));
}

#[tokio::test]
async fn test_register_existing_verified_email_returns_uniform_202_no_email_sent() {
    let harness = TestHarness::new().await;
    harness.clear_emails();

    let (_, _, _, _, _, _, email) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let payload = json!({
        "email": email,
        "password": "Password123!",
    });

    let (status, body) = harness.send_request("POST", "/auth/register", None, Some(payload)).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body["status"], "pending_verification");

    // No email sent
    let emails = harness.drain_emails().await;
    assert!(!emails.iter().any(|m| m.to == email));
}

#[tokio::test]
async fn test_register_existing_suspended_returns_uniform_202() {
    let harness = TestHarness::new().await;
    harness.clear_emails();

    let email = harness.create_user_with_status(UserStatus::Suspended).await;

    let payload = json!({
        "email": email,
        "password": "Password123!",
    });

    let (status, body) = harness.send_request("POST", "/auth/register", None, Some(payload)).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body["status"], "pending_verification");

    // No email sent
    let emails = harness.drain_emails().await;
    assert!(!emails.iter().any(|m| m.to == email));
}

#[tokio::test]
async fn test_register_existing_deleted_returns_uniform_202() {
    let harness = TestHarness::new().await;
    harness.clear_emails();

    let email = harness.create_user_with_status(UserStatus::Deleted).await;

    let payload = json!({
        "email": email,
        "password": "Password123!",
    });

    let (status, body) = harness.send_request("POST", "/auth/register", None, Some(payload)).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body["status"], "pending_verification");

    // No email sent
    let emails = harness.drain_emails().await;
    assert!(!emails.iter().any(|m| m.to == email));
}

#[tokio::test]
async fn test_register_response_body_byte_identical() {
    let harness = TestHarness::new().await;
    let (_, _, _, _, _, _, verified_email) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let new_email = format!("reg_byte_{}@has.ltd", Uuid::now_v7());

    let (_, body_new) = harness
        .send_request("POST", "/auth/register", None, Some(json!({"email": new_email, "password": "Password123!"})))
        .await;

    let (_, body_existing) = harness
        .send_request("POST", "/auth/register", None, Some(json!({"email": verified_email, "password": "Password123!"})))
        .await;

    let bytes_new = serde_json::to_vec(&body_new).unwrap();
    let bytes_existing = serde_json::to_vec(&body_existing).unwrap();
    assert_eq!(bytes_new, bytes_existing);
}


#[tokio::test]
async fn test_register_uniform_timing_across_three_paths() {
    let harness = TestHarness::new().await;

    // Setup existing verified user
    let (_, _, _, _, _, _, verified_email) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    // Setup existing pending user
    let pending_email = format!("pending_timing_{}@has.ltd", Uuid::now_v7());
    let _ = harness
        .send_request("POST", "/auth/register", None, Some(json!({"email": pending_email, "password": "Password123!"})))
        .await;

    // Warm up: 5 requests per path
    for _ in 0..5 {
        let warmup_new = format!("warmup_{}@has.ltd", Uuid::now_v7());
        let _ = harness.send_request("POST", "/auth/register", None, Some(json!({"email": warmup_new, "password": "Password123!"}))).await;
        let _ = harness.send_request("POST", "/auth/register", None, Some(json!({"email": verified_email, "password": "Password123!"}))).await;
        let _ = harness.send_request("POST", "/auth/register", None, Some(json!({"email": pending_email, "password": "Password123!"}))).await;
    }

    // Measure: 20 iterations per path
    let mut durations_new = Vec::with_capacity(20);
    let mut durations_verified = Vec::with_capacity(20);
    let mut durations_pending = Vec::with_capacity(20);

    for _ in 0..20 {
        let new_email = format!("timing_meas_{}@has.ltd", Uuid::now_v7());
        let t0 = Instant::now();
        let (s0, _) = harness.send_request("POST", "/auth/register", None, Some(json!({"email": new_email, "password": "Password123!"}))).await;
        durations_new.push(t0.elapsed().as_secs_f64());
        assert_eq!(s0, StatusCode::ACCEPTED);

        let t1 = Instant::now();
        let (s1, _) = harness.send_request("POST", "/auth/register", None, Some(json!({"email": verified_email, "password": "Password123!"}))).await;
        durations_verified.push(t1.elapsed().as_secs_f64());
        assert_eq!(s1, StatusCode::ACCEPTED);

        let t2 = Instant::now();
        let (s2, _) = harness.send_request("POST", "/auth/register", None, Some(json!({"email": pending_email, "password": "Password123!"}))).await;
        durations_pending.push(t2.elapsed().as_secs_f64());
        assert_eq!(s2, StatusCode::ACCEPTED);
    }

    let mean_new: f64 = durations_new.iter().sum::<f64>() / durations_new.len() as f64;
    let mean_verified: f64 = durations_verified.iter().sum::<f64>() / durations_verified.len() as f64;
    let mean_pending: f64 = durations_pending.iter().sum::<f64>() / durations_pending.len() as f64;

    let diff_new_ver = (mean_new - mean_verified).abs() / mean_new.max(mean_verified);
    let diff_new_pen = (mean_new - mean_pending).abs() / mean_new.max(mean_pending);
    let diff_ver_pen = (mean_verified - mean_pending).abs() / mean_verified.max(mean_pending);

    assert!(diff_new_ver < 0.15, "New vs Verified pairwise mean latency diff {:.2}% exceeds 15%", diff_new_ver * 100.0);
    assert!(diff_new_pen < 0.15, "New vs Pending pairwise mean latency diff {:.2}% exceeds 15%", diff_new_pen * 100.0);
    assert!(diff_ver_pen < 0.15, "Verified vs Pending pairwise mean latency diff {:.2}% exceeds 15%", diff_ver_pen * 100.0);
}

#[tokio::test]
async fn test_register_existing_pending_email_overwrites_otp() {
    let harness = TestHarness::new().await;
    harness.clear_emails();

    let email = format!("reg_pend_{}@has.ltd", Uuid::now_v7());
    let payload = json!({
        "email": email,
        "password": "Password123!",
    });

    // 1st register
    let (s1, _) = harness.send_request("POST", "/auth/register", None, Some(payload.clone())).await;
    assert_eq!(s1, StatusCode::ACCEPTED);
    let email1 = harness.drain_emails().await.into_iter().find(|m| m.to == email).unwrap();
    let code1 = email1.variables["code"].as_str().unwrap().to_string();

    // 2nd register with same pending email
    let (s2, _) = harness.send_request("POST", "/auth/register", None, Some(payload)).await;
    assert_eq!(s2, StatusCode::ACCEPTED);
    let email2 = harness.drain_emails().await.into_iter().find(|m| m.to == email).unwrap();
    let code2 = email2.variables["code"].as_str().unwrap().to_string();

    assert_ne!(code1, code2);

    // Old code fails, new code succeeds
    let (v_old, _) = harness
        .send_request("POST", "/auth/register/verify-otp", None, Some(json!({"email": email, "code": code1})))
        .await;
    assert_eq!(v_old, StatusCode::BAD_REQUEST);

    let (v_new, body_new) = harness
        .send_request("POST", "/auth/register/verify-otp", None, Some(json!({"email": email, "code": code2})))
        .await;
    assert_eq!(v_new, StatusCode::OK);
    assert_eq!(body_new["verified"], true);
}

#[tokio::test]
async fn test_register_weak_password_returns_400_validation_error() {
    let harness = TestHarness::new().await;
    let payload = json!({
        "email": "weak_pw@has.ltd",
        "password": "short"
    });
    let (status, body) = harness.send_request("POST", "/auth/register", None, Some(payload)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "validation_error");
}

#[tokio::test]
async fn test_register_invalid_email_returns_400_validation_error() {
    let harness = TestHarness::new().await;
    let payload = json!({
        "email": "not-a-valid-email",
        "password": "Password123!"
    });
    let (status, body) = harness.send_request("POST", "/auth/register", None, Some(payload)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "validation_error");
}

#[tokio::test]
async fn test_register_rate_limit_4th_attempt_same_ip_returns_429() {
    let harness = TestHarness::new().await;
    let unique_ip = format!("203.0.113.{}", Uuid::new_v4().as_fields().0 % 250 + 1);

    for i in 1..=3 {
        let email = format!("ip_rl_{}_{}@has.ltd", i, Uuid::now_v7());
        let payload = json!({"email": email, "password": "Password123!"});
        let (status, _, _) = harness
            .send_request_with_headers("POST", "/auth/register", None, Some(payload), &[("x-forwarded-for", &unique_ip)])
            .await;
        assert_eq!(status, StatusCode::ACCEPTED);
    }

    // 4th attempt from same IP
    let email4 = format!("ip_rl_4_{}@has.ltd", Uuid::now_v7());
    let payload4 = json!({"email": email4, "password": "Password123!"});
    let (status, body, headers) = harness
        .send_request_with_headers("POST", "/auth/register", None, Some(payload4), &[("x-forwarded-for", &unique_ip)])
        .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"]["code"], "rate_limited");
    assert!(headers.get("retry-after").is_some());
}

// ============================================================================
// 2. OTP VERIFICATION TESTS
// ============================================================================

#[tokio::test]
async fn test_verify_otp_correct_code_activates_user_and_deletes_key() {
    let harness = TestHarness::new().await;
    harness.clear_emails();

    let email = format!("verify_act_{}@has.ltd", Uuid::now_v7());
    let (s, _) = harness
        .send_request("POST", "/auth/register", None, Some(json!({"email": email, "password": "Password123!"})))
        .await;
    assert_eq!(s, StatusCode::ACCEPTED);

    let msg = harness.drain_emails().await.into_iter().find(|m| m.to == email).unwrap();
    let code = msg.variables["code"].as_str().unwrap();

    let (status, body) = harness
        .send_request("POST", "/auth/register/verify-otp", None, Some(json!({"email": email, "code": code})))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["verified"], true);

    // Verify DB row
    let row: (String, Option<chrono::DateTime<chrono::Utc>>) = sqlx::query_as(
        "SELECT status, email_verified_at FROM users WHERE email = $1",
    )
    .bind(&email)
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert_eq!(row.0, "ACTIVE");
    assert!(row.1.is_some());

    // Second submit fails (single-use)
    let (status2, body2) = harness
        .send_request("POST", "/auth/register/verify-otp", None, Some(json!({"email": email, "code": code})))
        .await;
    assert_eq!(status2, StatusCode::BAD_REQUEST);
    assert_eq!(body2["error"]["code"], "otp_expired");
}

#[tokio::test]
async fn test_verify_otp_wrong_code_returns_400_invalid_otp() {
    let harness = TestHarness::new().await;
    let email = format!("verify_wrong_{}@has.ltd", Uuid::now_v7());
    harness
        .send_request("POST", "/auth/register", None, Some(json!({"email": email, "password": "Password123!"})))
        .await;

    let (status, body) = harness
        .send_request("POST", "/auth/register/verify-otp", None, Some(json!({"email": email, "code": "000000"})))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_otp");
}

#[tokio::test]
async fn test_verify_otp_expired_code_returns_400_otp_expired() {
    let harness = TestHarness::new().await;
    let email = format!("verify_exp_{}@has.ltd", Uuid::now_v7());
    harness
        .send_request("POST", "/auth/register", None, Some(json!({"email": email, "password": "Password123!"})))
        .await;

    // Delete Redis OTP key
    let user_id: (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();

    let mut redis = harness.state.redis.clone().unwrap();
    let _: Result<(), _> = redis.del(format!("otp:email_verify:{}", user_id.0)).await;

    let (status, body) = harness
        .send_request("POST", "/auth/register/verify-otp", None, Some(json!({"email": email, "code": "123456"})))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "otp_expired");
}

#[tokio::test]
async fn test_verify_otp_fifth_failed_attempt_invalidates_otp() {
    let harness = TestHarness::new().await;
    let email = format!("verify_5th_{}@has.ltd", Uuid::now_v7());
    harness
        .send_request("POST", "/auth/register", None, Some(json!({"email": email, "password": "Password123!"})))
        .await;

    let user_id: (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();

    let otp_key = format!("otp:email_verify:{}", user_id.0);

    for i in 1..=4 {
        let (status, body) = harness
            .send_request("POST", "/auth/register/verify-otp", None, Some(json!({"email": email, "code": "000000"})))
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "Attempt {}", i);
        assert_eq!(body["error"]["code"], "invalid_otp");
    }

    // 5th failed attempt -> otp_expired and key deleted
    let (status5, body5) = harness
        .send_request("POST", "/auth/register/verify-otp", None, Some(json!({"email": email, "code": "000000"})))
        .await;
    assert_eq!(status5, StatusCode::BAD_REQUEST);
    assert_eq!(body5["error"]["code"], "otp_expired");

    let mut redis = harness.state.redis.clone().unwrap();
    let exists: bool = redis.exists(&otp_key).await.unwrap();
    assert!(!exists, "OTP key must be deleted after 5 failed attempts");
}

#[tokio::test]
async fn test_verify_otp_cross_user_isolation() {
    let harness = TestHarness::new().await;
    harness.clear_emails();

    let email_a = format!("user_a_{}@has.ltd", Uuid::now_v7());
    let email_b = format!("user_b_{}@has.ltd", Uuid::now_v7());

    harness
        .send_request("POST", "/auth/register", None, Some(json!({"email": email_a, "password": "Password123!"})))
        .await;
    harness
        .send_request("POST", "/auth/register", None, Some(json!({"email": email_b, "password": "Password123!"})))
        .await;

    let emails = harness.drain_emails().await;
    let msg_a = emails.iter().find(|m| m.to == email_a).unwrap();
    let code_a = msg_a.variables["code"].as_str().unwrap();

    // Use User A's code to verify User B
    let (status, body) = harness
        .send_request("POST", "/auth/register/verify-otp", None, Some(json!({"email": email_b, "code": code_a})))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_otp");
}

// ============================================================================
// 3. RESEND OTP TESTS
// ============================================================================

#[tokio::test]
async fn test_resend_otp_rate_limit_and_backoff_curve() {
    let harness = TestHarness::new().await;
    harness.clear_emails();

    let email = format!("resend_{}@has.ltd", Uuid::now_v7());
    harness
        .send_request("POST", "/auth/register", None, Some(json!({"email": email, "password": "Password123!"})))
        .await;

    let user_id: (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();

    // Immediate resend within 60s -> 429
    let (s_fail, body_fail, headers_fail) = harness
        .send_request_with_headers("POST", "/auth/register/resend-otp", None, Some(json!({"email": email})), &[])
        .await;
    assert_eq!(s_fail, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body_fail["error"]["code"], "rate_limited");
    assert!(headers_fail.get("retry-after").is_some());

    // Advance last send time by 65 seconds in Redis
    let mut redis = harness.state.redis.clone().unwrap();
    let past_time = chrono::Utc::now().timestamp() - 65;
    let _: Result<(), _> = redis.set(format!("rl:otp_send_last:{}", user_id.0), past_time).await;

    // Resend at 65s -> 200 OK
    let (s_ok, body_ok) = harness
        .send_request("POST", "/auth/register/resend-otp", None, Some(json!({"email": email})))
        .await;
    assert_eq!(s_ok, StatusCode::OK);
    assert_eq!(body_ok["status"], "dispatched");

    // Check counter incremented to 2
    let count: i64 = redis.get(format!("rl:otp_send_count:{}", user_id.0)).await.unwrap();
    assert_eq!(count, 2);
}

// ============================================================================
// 4. PASSWORD FORGOT TESTS
// ============================================================================

#[tokio::test]
async fn test_password_forgot_existing_vs_non_existing_uniformity() {
    let harness = TestHarness::new().await;
    harness.clear_emails();

    let (_, _, _, _, _, _, email) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let non_existing = format!("non_existing_{}@has.ltd", Uuid::now_v7());

    // 1. Existing email
    let (s1, b1) = harness
        .send_request("POST", "/auth/password/forgot", None, Some(json!({"email": email})))
        .await;
    assert_eq!(s1, StatusCode::ACCEPTED);

    let emails = harness.drain_emails().await;
    let captured = emails.into_iter().find(|m| m.to == email).expect("Forgot email must be captured");
    assert_eq!(captured.template_id, "password_reset_otp");

    // 2. Non-existing email
    let (s2, b2) = harness
        .send_request("POST", "/auth/password/forgot", None, Some(json!({"email": non_existing})))
        .await;
    assert_eq!(s2, StatusCode::ACCEPTED);

    // No email captured for non-existing
    let emails2 = harness.drain_emails().await;
    assert!(!emails2.iter().any(|m| m.to == non_existing));

    // Response body byte-identical
    let bytes1 = serde_json::to_vec(&b1).unwrap();
    let bytes2 = serde_json::to_vec(&b2).unwrap();
    assert_eq!(bytes1, bytes2);
}

#[tokio::test]
async fn test_password_forgot_rate_limits() {
    let harness = TestHarness::new().await;
    let unique_ip = format!("198.51.100.{}", Uuid::new_v4().as_fields().0 % 250 + 1);

    // Email rate limit: 2nd forgot for same email in 5m -> 429
    let email = format!("forgot_email_rl_{}@has.ltd", Uuid::now_v7());
    let (s1, _, _) = harness
        .send_request_with_headers("POST", "/auth/password/forgot", None, Some(json!({"email": email})), &[("x-forwarded-for", &unique_ip)])
        .await;
    assert_eq!(s1, StatusCode::ACCEPTED);

    let (s2, body2, _) = harness
        .send_request_with_headers("POST", "/auth/password/forgot", None, Some(json!({"email": email})), &[("x-forwarded-for", &unique_ip)])
        .await;
    assert_eq!(s2, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body2["error"]["code"], "rate_limited");

    // IP rate limit: 4th forgot from same IP in 1h -> 429
    let ip_for_4th = format!("198.51.101.{}", Uuid::new_v4().as_fields().0 % 250 + 1);
    for i in 1..=3 {
        let em = format!("forgot_ip_rl_{}_{}@has.ltd", i, Uuid::now_v7());
        let (s, _, _) = harness
            .send_request_with_headers("POST", "/auth/password/forgot", None, Some(json!({"email": em})), &[("x-forwarded-for", &ip_for_4th)])
            .await;
        assert_eq!(s, StatusCode::ACCEPTED);
    }
    let em4 = format!("forgot_ip_rl_4_{}@has.ltd", Uuid::now_v7());
    let (s4, body4, headers4) = harness
        .send_request_with_headers("POST", "/auth/password/forgot", None, Some(json!({"email": em4})), &[("x-forwarded-for", &ip_for_4th)])
        .await;
    assert_eq!(s4, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body4["error"]["code"], "rate_limited");
    assert!(headers4.get("retry-after").is_some());
}

// ============================================================================
// 5. PASSWORD RESET TESTS
// ============================================================================

#[tokio::test]
async fn test_password_reset_full_revocation_and_invalidation() {
    let harness = TestHarness::new().await;
    harness.clear_emails();

    let (user_id, session_id, refresh_id, _, raw_refresh, old_access_token, email) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    // Trigger forgot password
    harness
        .send_request("POST", "/auth/password/forgot", None, Some(json!({"email": email})))
        .await;

    let emails = harness.drain_emails().await;
    let msg = emails.iter().find(|m| m.to == email).unwrap();
    let code = msg.variables["code"].as_str().unwrap();

    // Verify token works before reset
    let (s_me, _) = harness.send_request("GET", "/auth/me", Some(&old_access_token), None).await;
    assert_eq!(s_me, StatusCode::OK);

    // Populate Redis auth_version cache
    let mut redis = harness.state.redis.clone().unwrap();
    let _: Result<(), _> = redis.set_ex(format!("auth_version:{}", user_id.into_inner()), 1, 300).await;

    // Reset password
    let new_password = "BrandNewPassword456!".to_string();
    let (s_reset, b_reset) = harness
        .send_request(
            "POST",
            "/auth/password/reset",
            None,
            Some(json!({
                "email": email,
                "code": code,
                "new_password": new_password
            })),
        )
        .await;
    assert_eq!(s_reset, StatusCode::OK);
    assert_eq!(b_reset["success"], true);

    // 1. Users.auth_version incremented
    let auth_version: (i64,) = sqlx::query_as("SELECT auth_version FROM users WHERE id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    assert_eq!(auth_version.0, 2);

    // 2. Credentials updated
    let cred: (String,) = sqlx::query_as("SELECT secret_hash FROM credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    assert!(crypto::password::verify_password(&new_password, &cred.0).unwrap());

    // 3. Sessions revoked
    let s_row: (Option<chrono::DateTime<chrono::Utc>>,) = sqlx::query_as("SELECT revoked_at FROM sessions WHERE id = $1")
        .bind(session_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    assert!(s_row.0.is_some());

    // 4. Refresh tokens revoked
    let r_row: (Option<chrono::DateTime<chrono::Utc>>,) = sqlx::query_as("SELECT revoked_at FROM refresh_tokens WHERE id = $1")
        .bind(refresh_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    assert!(r_row.0.is_some());

    // 5. Outstanding access token -> 401 token_revoked
    let (s_rev, b_rev) = harness.send_request("GET", "/auth/me", Some(&old_access_token), None).await;
    assert_eq!(s_rev, StatusCode::UNAUTHORIZED);
    assert_eq!(b_rev["error"]["code"], "token_revoked");

    // 6. Outstanding refresh token -> 401
    let cookie_header = format!("__Secure-has_refresh={}", raw_refresh);
    let (s_ref, _, _) = harness
        .send_request_with_headers("POST", "/auth/refresh", None, None, &[("cookie", &cookie_header)])
        .await;
    assert_eq!(s_ref, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_password_reset_wrong_otp_returns_400_invalid_otp() {
    let harness = TestHarness::new().await;
    let (_, _, _, _, _, _, email) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    harness
        .send_request("POST", "/auth/password/forgot", None, Some(json!({"email": email})))
        .await;

    let (status, body) = harness
        .send_request(
            "POST",
            "/auth/password/reset",
            None,
            Some(json!({
                "email": email,
                "code": "000000",
                "new_password": "ValidPassword123!"
            })),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_otp");
}

#[tokio::test]
async fn test_password_reset_weak_new_password_returns_400_validation_error() {
    let harness = TestHarness::new().await;
    let payload = json!({
        "email": "reset_weak@has.ltd",
        "code": "123456",
        "new_password": "short"
    });
    let (status, body) = harness.send_request("POST", "/auth/password/reset", None, Some(payload)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "validation_error");
}

// ============================================================================
// 6. PASSWORD CHANGE TESTS
// ============================================================================

async fn password_change_revokes_other_sessions_keeps_calling_session_alive_impl() {
    let harness = TestHarness::new().await;

    let (user_id, session1, _, _, _, token1, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    // Create a second session on another device
    let (session2, _, _, token2) = harness.add_extra_session(user_id).await;

    // Verify both tokens work
    let (s1, _) = harness.send_request("GET", "/auth/me", Some(&token1), None).await;
    assert_eq!(s1, StatusCode::OK);
    let (s2, _) = harness.send_request("GET", "/auth/me", Some(&token2), None).await;
    assert_eq!(s2, StatusCode::OK);

    // Call password change from session 1
    let payload = json!({
        "current_password": "Password123!",
        "new_password": "ChangedPassword456!"
    });

    let (status, body) = harness
        .send_request("POST", "/auth/password/change", Some(&token1), Some(payload))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);

    // Calling session received new access token
    let new_token1 = body["access_token"].as_str().expect("access_token must be present");

    // Critical assertion: Calling session's access token still works AFTER change
    let (s_new, body_me) = harness.send_request("GET", "/auth/me", Some(new_token1), None).await;
    assert_eq!(s_new, StatusCode::OK);
    assert_eq!(body_me["id"].as_str().unwrap(), user_id.into_inner().to_string());

    // Assert old calling access token is invalidated by the auth_version bump
    let (s_old, b_old) = harness.send_request("GET", "/auth/me", Some(&token1), None).await;
    assert_eq!(s_old, StatusCode::UNAUTHORIZED);
    assert_eq!(b_old["error"]["code"], "token_revoked");

    // Calling session in DB is ALIVE
    let s1_row: (Option<chrono::DateTime<chrono::Utc>>,) = sqlx::query_as("SELECT revoked_at FROM sessions WHERE id = $1")
        .bind(session1.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    assert!(s1_row.0.is_none(), "Calling session must NOT be revoked");

    // Other session is REVOKED
    let s2_row: (Option<chrono::DateTime<chrono::Utc>>,) = sqlx::query_as("SELECT revoked_at FROM sessions WHERE id = $1")
        .bind(session2.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    assert!(s2_row.0.is_some(), "Other session MUST be revoked");

    // Other device token is rejected
    let (s_other, b_other) = harness.send_request("GET", "/auth/me", Some(&token2), None).await;
    assert_eq!(s_other, StatusCode::UNAUTHORIZED);
    assert_eq!(b_other["error"]["code"], "token_revoked");
}

#[tokio::test]
async fn test_password_change_revokes_other_sessions_keeps_calling_session_alive() {
    password_change_revokes_other_sessions_keeps_calling_session_alive_impl().await;
}

#[tokio::test]
async fn test_password_change_revokes_other_sessions_keeps_calling_session_alive_via_successor_token() {
    let harness = TestHarness::new().await;

    let (user_id, session1, _, _, _, token1, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let payload = json!({
        "current_password": "Password123!",
        "new_password": "ChangedPassword456!"
    });

    let (status, body) = harness
        .send_request("POST", "/auth/password/change", Some(&token1), Some(payload))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);

    let new_token1 = body["access_token"].as_str().expect("access_token must be present");
    let claims = harness.state.jwt_keys.verify_access_token(new_token1).expect("valid access token");
    assert_eq!(claims.sid, session1.into_inner(), "claims.sid must match the calling session ID");
    assert_eq!(claims.sub, user_id.into_inner().to_string());
}

#[tokio::test]
async fn test_password_change_wrong_current_password_returns_401() {
    let harness = TestHarness::new().await;
    let (_, _, _, _, _, token, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let payload = json!({
        "current_password": "WrongPassword!",
        "new_password": "NewValidPassword123!"
    });

    let (status, body) = harness
        .send_request("POST", "/auth/password/change", Some(&token), Some(payload))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_credentials");
}

#[tokio::test]
async fn test_password_change_unauthenticated_returns_401() {
    let harness = TestHarness::new().await;
    let payload = json!({
        "current_password": "Password123!",
        "new_password": "NewValidPassword123!"
    });

    let (status, body) = harness
        .send_request("POST", "/auth/password/change", None, Some(payload))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_token");
}

// ============================================================================
// 7. CROSS-CUTTING AUDIT & CONTRACT TESTS
// ============================================================================

#[tokio::test]
async fn test_cross_cutting_audit_events_and_no_secrets_leaked() {
    let harness = TestHarness::new().await;
    harness.clear_emails();

    // 1. Register
    let reg_email = format!("audit_user_{}@has.ltd", Uuid::now_v7());
    harness
        .send_request("POST", "/auth/register", None, Some(json!({"email": reg_email, "password": "Password123!"})))
        .await;

    let emails = harness.drain_emails().await;
    let msg = emails.iter().find(|m| m.to == reg_email).unwrap();
    let code = msg.variables["code"].as_str().unwrap();

    // 2. Verify
    harness
        .send_request("POST", "/auth/register/verify-otp", None, Some(json!({"email": reg_email, "code": code})))
        .await;

    // 3. Forgot
    harness
        .send_request("POST", "/auth/password/forgot", None, Some(json!({"email": reg_email})))
        .await;

    let emails2 = harness.drain_emails().await;
    let msg2 = emails2.iter().find(|m| m.to == reg_email).unwrap();
    let reset_code = msg2.variables["code"].as_str().unwrap();

    // 4. Reset
    harness
        .send_request(
            "POST",
            "/auth/password/reset",
            None,
            Some(json!({"email": reg_email, "code": reset_code, "new_password": "NewPassword456!"})),
        )
        .await;

    // Check all audit rows created
    let rows: Vec<(String, Value)> = sqlx::query_as("SELECT event, metadata FROM audit_log ORDER BY created_at DESC LIMIT 50")
        .fetch_all(&harness.state.db_pool)
        .await
        .unwrap();

    let events: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();
    assert!(events.contains(&"auth.register.requested"));
    assert!(events.contains(&"auth.register.verified"));
    assert!(events.contains(&"auth.password.reset_requested"));
    assert!(events.contains(&"auth.password.reset_completed"));

    // Assert that NO metadata row contains secret keywords or OTP codes
    for (event, meta) in &rows {
        let meta_str = meta.to_string().to_lowercase();
        assert!(!meta_str.contains("password123"), "Secret password leaked in audit row for {}", event);
        assert!(!meta_str.contains("argon2"), "Password hash leaked in audit row for {}", event);
        assert!(!meta_str.contains(code), "OTP code leaked in audit row for {}", event);
    }
}

#[tokio::test]
async fn test_verify_otp_rate_limit_6th_attempt_in_10m_returns_429() {
    let harness = TestHarness::new().await;
    let email = format!("verify_6th_{}@has.ltd", Uuid::now_v7());
    harness
        .send_request("POST", "/auth/register", None, Some(json!({"email": email, "password": "Password123!"})))
        .await;

    // Attempts 1 to 4 -> 400 invalid_otp
    for _ in 1..=4 {
        let (s, b) = harness
            .send_request("POST", "/auth/register/verify-otp", None, Some(json!({"email": email, "code": "000000"})))
            .await;
        assert_eq!(s, StatusCode::BAD_REQUEST);
        assert_eq!(b["error"]["code"], "invalid_otp");
    }

    // 5th attempt -> 400 otp_expired (OTP key invalidated)
    let (s5, b5) = harness
        .send_request("POST", "/auth/register/verify-otp", None, Some(json!({"email": email, "code": "000000"})))
        .await;
    assert_eq!(s5, StatusCode::BAD_REQUEST);
    assert_eq!(b5["error"]["code"], "otp_expired");

    // 6th attempt -> 429 rate_limited with Retry-After header
    let (s6, b6, h6) = harness
        .send_request_with_headers("POST", "/auth/register/verify-otp", None, Some(json!({"email": email, "code": "000000"})), &[])
        .await;
    assert_eq!(s6, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(b6["error"]["code"], "rate_limited");
    assert!(h6.get("retry-after").is_some());
}

#[tokio::test]
async fn test_verify_otp_second_submit_with_same_code_returns_400() {
    let harness = TestHarness::new().await;
    harness.clear_emails();

    let email = format!("verify_twice_{}@has.ltd", Uuid::now_v7());
    harness
        .send_request("POST", "/auth/register", None, Some(json!({"email": email, "password": "Password123!"})))
        .await;

    let emails = harness.drain_emails().await;
    let msg = emails.iter().find(|m| m.to == email).unwrap();
    let code = msg.variables["code"].as_str().unwrap();

    // 1st verify -> 200 OK
    let (s1, b1) = harness
        .send_request("POST", "/auth/register/verify-otp", None, Some(json!({"email": email, "code": code})))
        .await;
    assert_eq!(s1, StatusCode::OK);
    assert_eq!(b1["verified"], true);

    // 2nd verify with same code -> 400 otp_expired (key deleted)
    let (s2, b2) = harness
        .send_request("POST", "/auth/register/verify-otp", None, Some(json!({"email": email, "code": code})))
        .await;
    assert_eq!(s2, StatusCode::BAD_REQUEST);
    assert_eq!(b2["error"]["code"], "otp_expired");
}

#[tokio::test]
async fn test_error_response_matches_phase3_shape() {
    let harness = TestHarness::new().await;
    let (s, b) = harness
        .send_request("POST", "/auth/register", None, Some(json!({"email": "bad-email", "password": "short"})))
        .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    assert!(b.get("error").is_some(), "Must have top-level error object");
    assert!(b["error"].get("code").is_some(), "Must have error.code");
    assert!(b["error"].get("message").is_some(), "Must have error.message");
}

#[tokio::test]
async fn test_password_change_audit_event_recorded() {
    let harness = TestHarness::new().await;
    let (user_id, session_id, _, _, _, token, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let (s, _) = harness
        .send_request(
            "POST",
            "/auth/password/change",
            Some(&token),
            Some(json!({"current_password": "Password123!", "new_password": "NewSecretPassword456!"})),
        )
        .await;
    assert_eq!(s, StatusCode::OK);

    let row: (String, Value) = sqlx::query_as(
        "SELECT event, metadata FROM audit_log WHERE user_id = $1 AND event = 'auth.password.changed'",
    )
    .bind(user_id.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();

    assert_eq!(row.0, "auth.password.changed");
    assert_eq!(row.1["session_id"], session_id.into_inner().to_string());
    let meta_str = row.1.to_string().to_lowercase();
    assert!(!meta_str.contains("password123"));
    assert!(!meta_str.contains("newsecretpassword"));
    assert!(!meta_str.contains("argon2"));
}

#[tokio::test]
async fn test_register_otp_hash_stored_in_redis_with_ttl_600() {
    let harness = TestHarness::new().await;
    let email = format!("ttl_check_{}@has.ltd", Uuid::now_v7());
    let (status, _) = harness
        .send_request("POST", "/auth/register", None, Some(json!({"email": email, "password": "Password123!"})))
        .await;
    assert_eq!(status, StatusCode::ACCEPTED);

    let user_id: (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();

    let mut redis = harness.state.redis.clone().unwrap();
    let ttl: i64 = redis::cmd("TTL")
        .arg(format!("otp:email_verify:{}", user_id.0))
        .query_async(&mut redis)
        .await
        .unwrap();

    assert!(ttl > 500 && ttl <= 600, "OTP key TTL must be near 600s, got {}", ttl);
}

#[tokio::test]
async fn test_register_in_memory_adapter_captured_one_email_details() {
    let harness = TestHarness::new().await;
    harness.clear_emails();

    let email = format!("email_msg_{}@has.ltd", Uuid::now_v7());
    let (status, _) = harness
        .send_request("POST", "/auth/register", None, Some(json!({"email": email, "password": "Password123!"})))
        .await;
    assert_eq!(status, StatusCode::ACCEPTED);

    let emails = harness.drain_emails().await;
    let matching: Vec<&EmailMessage> = emails.iter().filter(|m| m.to == email).collect();
    assert_eq!(matching.len(), 1, "Exactly one email message must be captured");

    let msg = matching[0];
    assert_eq!(msg.template_id, "email_verify_otp");
    assert_eq!(msg.subject, "Verify your email");
    let code = msg.variables["code"].as_str().expect("code string in variables");
    assert_eq!(code.len(), 6, "OTP code must be 6 digits");
    assert!(code.chars().all(|c| c.is_ascii_digit()), "OTP code must be all digits");

    // Submit extracted code to /auth/register/verify-otp and assert 200 OK per [TEST-3]
    let (v_status, v_body) = harness
        .send_request("POST", "/auth/register/verify-otp", None, Some(json!({"email": email, "code": code})))
        .await;
    assert_eq!(v_status, StatusCode::OK);
    assert_eq!(v_body["verified"], true);
}

#[tokio::test]
async fn test_password_reset_wrong_otp_does_not_mutate_credentials_or_sessions() {
    let harness = TestHarness::new().await;
    let (user_id, session_id, _, _, _, token, email) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    // Get old secret hash
    let (old_secret_hash,): (String,) = sqlx::query_as(
        "SELECT secret_hash FROM credentials WHERE user_id = $1",
    )
    .bind(user_id.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();

    let (s, b) = harness
        .send_request(
            "POST",
            "/auth/password/reset",
            None,
            Some(json!({"email": email, "code": "999999", "new_password": "FailedNewPassword123!"})),
        )
        .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    assert_eq!(b["error"]["code"], "invalid_otp");

    // Assert secret hash unchanged
    let (current_secret_hash,): (String,) = sqlx::query_as(
        "SELECT secret_hash FROM credentials WHERE user_id = $1",
    )
    .bind(user_id.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert_eq!(old_secret_hash, current_secret_hash);

    // Assert session still valid
    let (s_me, _) = harness.send_request("GET", "/auth/me", Some(&token), None).await;
    assert_eq!(s_me, StatusCode::OK);

    let (session_revoked,): (Option<chrono::DateTime<chrono::Utc>>,) = sqlx::query_as(
        "SELECT revoked_at FROM sessions WHERE id = $1",
    )
    .bind(session_id.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();
    assert!(session_revoked.is_none(), "Session must not be revoked on failed reset");
}

#[tokio::test]
async fn test_register_new_user_id_is_uuid_v7() {
    let harness = TestHarness::new().await;
    let email = format!("uuidv7_{}@example.com", Uuid::now_v7());

    let (status, _) = harness
        .send_request(
            "POST",
            "/auth/register",
            None,
            Some(json!({
                "email": email,
                "password": "Password123!",
                "name": "UUIDv7 Test User",
            })),
        )
        .await;
    assert_eq!(status, StatusCode::ACCEPTED);

    // Read user id from DB
    let (user_id,): (Uuid,) = sqlx::query_as(
        "SELECT id FROM users WHERE email_normalized = $1",
    )
    .bind(email.to_lowercase())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();

    assert_eq!(user_id.get_version_num(), 7);
}


