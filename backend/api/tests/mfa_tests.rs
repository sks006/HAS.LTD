// backend/api/tests/mfa_tests.rs
//! Phase 6B: Multi-Factor Authentication (TOTP & Recovery) Integration Tests
//! Minimum 30 tests covering:
//! - POST /auth/mfa/totp/enroll
//! - POST /auth/mfa/totp/verify
//! - POST /auth/mfa/totp/disable
//! - POST /auth/login/totp
//! - POST /auth/login/recovery
//! - Login MFA branching (Customer, Moderator, Admin)
//! - Single-use Redis nonce enforcement and replay prevention
//! - Zero secrets in audit logs

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

    async fn create_user_with_session(
        &self,
        role: UserRole,
        status: UserStatus,
        auth_version: i64,
    ) -> (UserId, SessionId, RefreshTokenId, Uuid, String, String, String) {
        let user_id = UserId::now_v7();
        let email = format!("test_mfa_{}@has.ltd", Uuid::now_v7());
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
        .bind("Test MFA User")
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
        let refresh_id = RefreshTokenId::now_v7();
        let family_id = Uuid::new_v4();
        let (raw_refresh, token_hash) = crypto::token::generate_refresh_token();
        let expires_at = now + chrono::Duration::days(30);

        sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, device_fingerprint, user_agent, ip_address, created_at, last_seen_at, expires_at, revoked_at)
            VALUES ($1, $2, NULL, 'test-agent', '127.0.0.1'::inet, $3, $3, $4, NULL)
            "#,
        )
        .bind(session_id.into_inner())
        .bind(user_id.into_inner())
        .bind(now)
        .bind(expires_at)
        .execute(&self.state.db_pool)
        .await
        .unwrap();

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
        .execute(&self.state.db_pool)
        .await
        .unwrap();

        let token = self.issue_token(user_id, role, auth_version, "access", session_id);
        (user_id, session_id, refresh_id, family_id, raw_refresh, token, email)
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

    fn issue_mfa_token(&self, user_id: UserId, role: UserRole, auth_version: i64, jti: Uuid) -> String {
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: user_id.into_inner().to_string(),
            iss: DEFAULT_ISSUER.to_string(),
            aud: DEFAULT_AUDIENCE.to_string(),
            exp: now + 300,
            iat: now,
            nbf: now,
            jti,
            sid: Uuid::nil(),
            auth_version,
            role: role.to_string(),
            scope: "mfa_pending".to_string(),
        };
        self.state.jwt_keys.sign_access_token(&claims).unwrap()
    }
}

// ============================================================================
// 1. ENROLLMENT TESTS
// ============================================================================

#[tokio::test]
async fn test_mfa_enroll_with_correct_password_returns_200_and_codes() {
    let harness = TestHarness::new().await;
    let (_, _, _, _, _, token, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let payload = json!({ "current_password": "Password123!" });
    let (status, body) = harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(payload))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["otpauth_url"].as_str().unwrap().starts_with("otpauth://totp/"));
    assert!(!body["secret"].as_str().unwrap().is_empty());
    let codes = body["recovery_codes"].as_array().expect("recovery codes array");
    assert_eq!(codes.len(), 10);
    for code in codes {
        assert_eq!(code.as_str().unwrap().len(), 10);
    }
}

#[tokio::test]
async fn test_mfa_enroll_with_wrong_password_returns_401() {
    let harness = TestHarness::new().await;
    let (_, _, _, _, _, token, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let payload = json!({ "current_password": "WrongPassword123!" });
    let (status, body) = harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(payload))
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_credentials");
}

#[tokio::test]
async fn test_mfa_enroll_without_auth_user_returns_401() {
    let harness = TestHarness::new().await;
    let payload = json!({ "current_password": "Password123!" });
    let (status, body) = harness
        .send_request("POST", "/auth/mfa/totp/enroll", None, Some(payload))
        .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_token");
}

#[tokio::test]
async fn test_mfa_enroll_stores_encrypted_secret() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let payload = json!({ "current_password": "Password123!" });
    let (status, body) = harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(payload))
        .await;
    assert_eq!(status, StatusCode::OK);

    let secret_str = body["secret"].as_str().unwrap();

    let row: (Vec<u8>,) = sqlx::query_as("SELECT secret_encrypted FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();

    // Verify DB does not contain plaintext secret string
    assert_ne!(row.0, secret_str.as_bytes());
    // Decrypt and confirm it decrypts to the valid secret
    let decrypted = crypto::totp::decrypt_totp_secret(&row.0, &harness.state.totp_key).expect("decryption succeeds");
    assert_eq!(decrypted.len(), 20);
}

#[tokio::test]
async fn test_mfa_enroll_stores_recovery_codes_as_argon2_hashes() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let payload = json!({ "current_password": "Password123!" });
    let (status, _body) = harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(payload))
        .await;
    assert_eq!(status, StatusCode::OK);

    let row: (Vec<String>,) = sqlx::query_as("SELECT recovery_codes_hash FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();

    assert_eq!(row.0.len(), 10);
    for hash in &row.0 {
        assert!(hash.starts_with("$argon2id$"));
    }
}

#[tokio::test]
async fn test_mfa_enroll_sets_confirmed_at_null() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let payload = json!({ "current_password": "Password123!" });
    let (status, _) = harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(payload))
        .await;
    assert_eq!(status, StatusCode::OK);

    let row: (Option<chrono::DateTime<chrono::Utc>>,) = sqlx::query_as("SELECT confirmed_at FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();

    assert!(row.0.is_none(), "confirmed_at must be NULL upon enrollment");
}

#[tokio::test]
async fn test_enroll_with_existing_confirmed_mfa_returns_409() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;
    let pwd = "Password123!";

    // Seed confirmed MFA
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
    .bind(user_id.into_inner())
    .bind(&secret_enc)
    .bind(&hashes)
    .execute(&harness.state.db_pool)
    .await
    .unwrap();

    let (old_enc, old_confirmed_at): (Vec<u8>, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        "SELECT secret_encrypted, confirmed_at FROM mfa_credentials WHERE user_id = $1",
    )
    .bind(user_id.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();

    // Attempt to re-enroll
    let (status, body) = harness
        .send_request(
            "POST",
            "/auth/mfa/totp/enroll",
            Some(&token),
            Some(json!({ "current_password": pwd })),
        )
        .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "mfa_already_enrolled");
    assert_eq!(
        body["error"]["message"],
        "MFA is already enabled. Disable it before re-enrolling."
    );

    // Verify original secret_encrypted and confirmed_at are unchanged
    let (cur_enc, cur_confirmed_at): (Vec<u8>, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        "SELECT secret_encrypted, confirmed_at FROM mfa_credentials WHERE user_id = $1",
    )
    .bind(user_id.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();

    assert_eq!(old_enc, cur_enc);
    assert_eq!(old_confirmed_at, cur_confirmed_at);
}

#[tokio::test]
async fn test_mfa_enroll_rate_limit() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    if let Some(mut redis) = harness.state.redis.clone() {
        let rl_key = format!("rl:mfa_enroll:{}", user_id.into_inner());
        let _: Result<(), _> = redis.set_ex(&rl_key, 5, 900).await;
    }

    let payload = json!({ "current_password": "Password123!" });
    let (status, body) = harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(payload))
        .await;

    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"]["code"], "rate_limited");
}

// ============================================================================
// 2. VERIFY ENROLLMENT TESTS
// ============================================================================

#[tokio::test]
async fn test_mfa_verify_correct_totp_code_confirms_enrollment() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    // Enroll
    let (s_enr, _b_enr) = harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(json!({ "current_password": "Password123!" })))
        .await;
    assert_eq!(s_enr, StatusCode::OK);

    // Read stored secret to generate valid code
    let row: (Vec<u8>,) = sqlx::query_as("SELECT secret_encrypted FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    let secret = crypto::totp::decrypt_totp_secret(&row.0, &harness.state.totp_key).unwrap();
    let totp = crypto::totp::create_totp_instance(&secret).unwrap();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let code = totp.generate(now);

    let (s_ver, b_ver) = harness
        .send_request("POST", "/auth/mfa/totp/verify", Some(&token), Some(json!({ "code": code })))
        .await;
    assert_eq!(s_ver, StatusCode::OK);
    assert_eq!(b_ver["success"], true);

    // Verify DB confirmed_at is now populated
    let row_conf: (Option<chrono::DateTime<chrono::Utc>>,) = sqlx::query_as("SELECT confirmed_at FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    assert!(row_conf.0.is_some(), "confirmed_at must be populated after verify");
}

#[tokio::test]
async fn test_mfa_verify_wrong_totp_code_returns_401() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(json!({ "current_password": "Password123!" })))
        .await;

    let (status, body) = harness
        .send_request("POST", "/auth/mfa/totp/verify", Some(&token), Some(json!({ "code": "000000" })))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "mfa_invalid");

    let row: (Option<chrono::DateTime<chrono::Utc>>,) = sqlx::query_as("SELECT confirmed_at FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    assert!(row.0.is_none(), "confirmed_at must remain NULL after failed verify");
}

#[tokio::test]
async fn test_mfa_verify_expired_enrollment_returns_404() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(json!({ "current_password": "Password123!" })))
        .await;

    // Simulate 31 minutes passed
    let past = chrono::Utc::now() - chrono::Duration::minutes(31);
    sqlx::query("UPDATE mfa_credentials SET created_at = $1 WHERE user_id = $2")
        .bind(past)
        .bind(user_id.into_inner())
        .execute(&harness.state.db_pool)
        .await
        .unwrap();

    let (status, body) = harness
        .send_request("POST", "/auth/mfa/totp/verify", Some(&token), Some(json!({ "code": "123456" })))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "enrollment_expired");
}

// ============================================================================
// 3. DISABLE TESTS
// ============================================================================

#[tokio::test]
async fn test_mfa_disable_correct_password_and_totp_deletes_row() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    // Enroll + Verify
    harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(json!({ "current_password": "Password123!" })))
        .await;

    let row: (Vec<u8>,) = sqlx::query_as("SELECT secret_encrypted FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    let secret = crypto::totp::decrypt_totp_secret(&row.0, &harness.state.totp_key).unwrap();
    let totp = crypto::totp::create_totp_instance(&secret).unwrap();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let code = totp.generate(now);

    harness
        .send_request("POST", "/auth/mfa/totp/verify", Some(&token), Some(json!({ "code": code })))
        .await;

    // Disable using fresh TOTP code
    let code_disable = totp.generate(now);
    let payload = json!({
        "current_password": "Password123!",
        "code": code_disable,
    });
    let (s_dis, b_dis) = harness
        .send_request("POST", "/auth/mfa/totp/disable", Some(&token), Some(payload))
        .await;
    assert_eq!(s_dis, StatusCode::OK);
    assert_eq!(b_dis["success"], true);

    // Row deleted
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    assert_eq!(count.0, 0);
}

#[tokio::test]
async fn test_mfa_disable_wrong_password_returns_401() {
    let harness = TestHarness::new().await;
    let (_, _, _, _, _, token, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let payload = json!({
        "current_password": "WrongPassword!",
        "code": "123456",
    });
    let (status, body) = harness
        .send_request("POST", "/auth/mfa/totp/disable", Some(&token), Some(payload))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_credentials");
}

#[tokio::test]
async fn test_mfa_disable_correct_password_and_recovery_code_succeeds() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    // Enroll and capture recovery codes
    let (_, b_enr) = harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(json!({ "current_password": "Password123!" })))
        .await;
    let recovery_code = b_enr["recovery_codes"][0].as_str().unwrap().to_string();

    // Verify enrollment
    let row: (Vec<u8>,) = sqlx::query_as("SELECT secret_encrypted FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    let secret = crypto::totp::decrypt_totp_secret(&row.0, &harness.state.totp_key).unwrap();
    let totp = crypto::totp::create_totp_instance(&secret).unwrap();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    harness
        .send_request("POST", "/auth/mfa/totp/verify", Some(&token), Some(json!({ "code": totp.generate(now) })))
        .await;

    // Disable using recovery code
    let payload = json!({
        "current_password": "Password123!",
        "code": recovery_code,
    });
    let (status, body) = harness
        .send_request("POST", "/auth/mfa/totp/disable", Some(&token), Some(payload))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    assert_eq!(count.0, 0);
}

// ============================================================================
// 4. LOGIN (NO MFA) TESTS
// ============================================================================

#[tokio::test]
async fn test_login_customer_without_mfa_logs_in_normally() {
    let harness = TestHarness::new().await;
    let (_, _, _, _, _, _, email) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let payload = json!({
        "email": email,
        "password": "Password123!",
    });
    let (status, body, headers) = harness
        .send_request_with_headers("POST", "/auth/login", None, Some(payload), &[])
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["mfa_required"], false);
    assert!(body["access_token"].as_str().is_some());
    assert!(headers.get(header::SET_COOKIE).is_some());
}

#[tokio::test]
async fn test_login_customer_with_confirmed_mfa_gets_challenge() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, email) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    // Enroll and confirm MFA
    harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(json!({ "current_password": "Password123!" })))
        .await;
    let row: (Vec<u8>,) = sqlx::query_as("SELECT secret_encrypted FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    let secret = crypto::totp::decrypt_totp_secret(&row.0, &harness.state.totp_key).unwrap();
    let totp = crypto::totp::create_totp_instance(&secret).unwrap();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    harness
        .send_request("POST", "/auth/mfa/totp/verify", Some(&token), Some(json!({ "code": totp.generate(now) })))
        .await;

    // Now log in via /auth/login
    let payload = json!({
        "email": email,
        "password": "Password123!",
    });
    let (status, body) = harness.send_request("POST", "/auth/login", None, Some(payload)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["mfa_required"], true);
    assert!(body["mfa_token"].as_str().is_some());
    assert!(body["access_token"].is_null());
}

#[tokio::test]
async fn test_login_moderator_without_mfa_returns_200_enrollment_token() {
    let harness = TestHarness::new().await;
    let (_, _, _, _, _, _, email) = harness
        .create_user_with_session(UserRole::Moderator, UserStatus::Active, 1)
        .await;

    let payload = json!({
        "email": email,
        "password": "Password123!",
    });
    let (status, body) = harness.send_request("POST", "/auth/login", None, Some(payload)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["mfa_enrollment_required"], true);
    assert_eq!(body["expires_in"], 900);
    assert!(body["enrollment_token"].as_str().is_some());
}

#[tokio::test]
async fn test_login_admin_without_mfa_returns_200_enrollment_token() {
    let harness = TestHarness::new().await;
    let (_, _, _, _, _, _, email) = harness
        .create_user_with_session(UserRole::Admin, UserStatus::Active, 1)
        .await;

    let payload = json!({
        "email": email,
        "password": "Password123!",
    });
    let (status, body) = harness.send_request("POST", "/auth/login", None, Some(payload)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["mfa_enrollment_required"], true);
    assert_eq!(body["expires_in"], 900);
    assert!(body["enrollment_token"].as_str().is_some());
}

// ============================================================================
// 5. LOGIN TOTP TESTS
// ============================================================================

#[tokio::test]
async fn test_login_totp_valid_token_and_valid_code_issues_session() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, email) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    // Enroll & confirm MFA
    harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(json!({ "current_password": "Password123!" })))
        .await;
    let row: (Vec<u8>,) = sqlx::query_as("SELECT secret_encrypted FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    let secret = crypto::totp::decrypt_totp_secret(&row.0, &harness.state.totp_key).unwrap();
    let totp = crypto::totp::create_totp_instance(&secret).unwrap();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    harness
        .send_request("POST", "/auth/mfa/totp/verify", Some(&token), Some(json!({ "code": totp.generate(now) })))
        .await;

    // Get mfa_token from login
    let (_, b_login) = harness
        .send_request("POST", "/auth/login", None, Some(json!({ "email": email, "password": "Password123!" })))
        .await;
    let mfa_token = b_login["mfa_token"].as_str().unwrap();

    // Submit valid TOTP
    let code = totp.generate(now);
    let (status, body, headers) = harness
        .send_request_with_headers("POST", "/auth/login/totp", None, Some(json!({ "mfa_token": mfa_token, "code": code })), &[])
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["mfa_required"], false);
    assert!(body["access_token"].as_str().is_some());
    assert!(headers.get(header::SET_COOKIE).is_some());
}

#[tokio::test]
async fn test_login_totp_valid_token_and_wrong_code_returns_401() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, email) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(json!({ "current_password": "Password123!" })))
        .await;
    let row: (Vec<u8>,) = sqlx::query_as("SELECT secret_encrypted FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    let secret = crypto::totp::decrypt_totp_secret(&row.0, &harness.state.totp_key).unwrap();
    let totp = crypto::totp::create_totp_instance(&secret).unwrap();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    harness
        .send_request("POST", "/auth/mfa/totp/verify", Some(&token), Some(json!({ "code": totp.generate(now) })))
        .await;

    let (_, b_login) = harness
        .send_request("POST", "/auth/login", None, Some(json!({ "email": email, "password": "Password123!" })))
        .await;
    let mfa_token = b_login["mfa_token"].as_str().unwrap();

    let (status, body) = harness
        .send_request("POST", "/auth/login/totp", None, Some(json!({ "mfa_token": mfa_token, "code": "000000" })))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "mfa_invalid");
}

#[tokio::test]
async fn test_login_totp_malformed_token_returns_401_invalid_token() {
    let harness = TestHarness::new().await;
    let (status, body) = harness
        .send_request("POST", "/auth/login/totp", None, Some(json!({ "mfa_token": "invalid.jwt.token", "code": "123456" })))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_token");
}

#[tokio::test]
async fn test_login_totp_expired_token_returns_401_challenge_expired() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, _, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let past = chrono::Utc::now().timestamp() - 100;
    let jti = Uuid::now_v7();
    let claims = Claims {
        sub: user_id.into_inner().to_string(),
        iss: DEFAULT_ISSUER.to_string(),
        aud: DEFAULT_AUDIENCE.to_string(),
        exp: past,
        iat: past - 300,
        nbf: past - 300,
        jti,
        sid: Uuid::nil(),
        auth_version: 1,
        role: "CUSTOMER".to_string(),
        scope: "mfa_pending".to_string(),
    };
    let expired_token = harness.state.jwt_keys.sign_access_token(&claims).unwrap();

    let (status, body) = harness
        .send_request("POST", "/auth/login/totp", None, Some(json!({ "mfa_token": expired_token, "code": "123456" })))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "mfa_challenge_expired");
}

#[tokio::test]
async fn test_login_totp_replayed_token_returns_401_challenge_expired() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, email) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(json!({ "current_password": "Password123!" })))
        .await;
    let row: (Vec<u8>,) = sqlx::query_as("SELECT secret_encrypted FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    let secret = crypto::totp::decrypt_totp_secret(&row.0, &harness.state.totp_key).unwrap();
    let totp = crypto::totp::create_totp_instance(&secret).unwrap();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    harness
        .send_request("POST", "/auth/mfa/totp/verify", Some(&token), Some(json!({ "code": totp.generate(now) })))
        .await;

    let (_, b_login) = harness
        .send_request("POST", "/auth/login", None, Some(json!({ "email": email, "password": "Password123!" })))
        .await;
    let mfa_token = b_login["mfa_token"].as_str().unwrap();
    let code = totp.generate(now);

    // 1st consumption succeeds
    let (s1, _) = harness
        .send_request("POST", "/auth/login/totp", None, Some(json!({ "mfa_token": mfa_token, "code": code })))
        .await;
    assert_eq!(s1, StatusCode::OK);

    // 2nd consumption must fail as challenge was deleted atomically
    let (s2, b2) = harness
        .send_request("POST", "/auth/login/totp", None, Some(json!({ "mfa_token": mfa_token, "code": code })))
        .await;
    assert_eq!(s2, StatusCode::UNAUTHORIZED);
    assert_eq!(b2["error"]["code"], "mfa_challenge_expired");
}

#[tokio::test]
async fn test_login_totp_rate_limit() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, _, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let jti = Uuid::now_v7();
    if let Some(mut redis) = harness.state.redis.clone() {
        let challenge_key = format!("mfa:challenge:{}", jti);
        let _: Result<(), _> = redis.set_ex(&challenge_key, user_id.into_inner().to_string(), 300).await;
        let rl_key = format!("rl:mfa_login:{}", user_id.into_inner());
        let _: Result<(), _> = redis.set_ex(&rl_key, 5, 60).await;
    }

    let mfa_token = harness.issue_mfa_token(user_id, UserRole::Customer, 1, jti);
    let (status, body) = harness
        .send_request("POST", "/auth/login/totp", None, Some(json!({ "mfa_token": mfa_token, "code": "123456" })))
        .await;

    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"]["code"], "rate_limited");
}

// ============================================================================
// 6. LOGIN RECOVERY TESTS
// ============================================================================

#[tokio::test]
async fn test_login_recovery_valid_token_and_valid_code_issues_session() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, email) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    // Enroll and capture recovery codes
    let (_, b_enr) = harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(json!({ "current_password": "Password123!" })))
        .await;
    let recovery_code = b_enr["recovery_codes"][0].as_str().unwrap().to_string();

    // Confirm enrollment
    let row: (Vec<u8>,) = sqlx::query_as("SELECT secret_encrypted FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    let secret = crypto::totp::decrypt_totp_secret(&row.0, &harness.state.totp_key).unwrap();
    let totp = crypto::totp::create_totp_instance(&secret).unwrap();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    harness
        .send_request("POST", "/auth/mfa/totp/verify", Some(&token), Some(json!({ "code": totp.generate(now) })))
        .await;

    // Login to get challenge
    let (_, b_login) = harness
        .send_request("POST", "/auth/login", None, Some(json!({ "email": email, "password": "Password123!" })))
        .await;
    let mfa_token = b_login["mfa_token"].as_str().unwrap();

    let (status, body, headers) = harness
        .send_request_with_headers("POST", "/auth/login/recovery", None, Some(json!({ "mfa_token": mfa_token, "recovery_code": recovery_code })), &[])
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["mfa_required"], false);
    assert!(body["access_token"].as_str().is_some());
    assert!(headers.get(header::SET_COOKIE).is_some());
}

#[tokio::test]
async fn test_login_recovery_deletes_used_code_from_array() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, email) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let (_, b_enr) = harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(json!({ "current_password": "Password123!" })))
        .await;
    let recovery_code = b_enr["recovery_codes"][0].as_str().unwrap().to_string();

    let row: (Vec<u8>,) = sqlx::query_as("SELECT secret_encrypted FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    let secret = crypto::totp::decrypt_totp_secret(&row.0, &harness.state.totp_key).unwrap();
    let totp = crypto::totp::create_totp_instance(&secret).unwrap();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    harness
        .send_request("POST", "/auth/mfa/totp/verify", Some(&token), Some(json!({ "code": totp.generate(now) })))
        .await;

    let (_, b_login) = harness
        .send_request("POST", "/auth/login", None, Some(json!({ "email": email, "password": "Password123!" })))
        .await;
    let mfa_token = b_login["mfa_token"].as_str().unwrap();

    let (status, _) = harness
        .send_request("POST", "/auth/login/recovery", None, Some(json!({ "mfa_token": mfa_token, "recovery_code": recovery_code })))
        .await;
    assert_eq!(status, StatusCode::OK);

    // Verify remaining recovery codes is now 9
    let row_rem: (Vec<String>,) = sqlx::query_as("SELECT recovery_codes_hash FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    assert_eq!(row_rem.0.len(), 9);
}

#[tokio::test]
async fn test_login_recovery_second_use_of_same_code_fails() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, email) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let (_, b_enr) = harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(json!({ "current_password": "Password123!" })))
        .await;
    let recovery_code = b_enr["recovery_codes"][0].as_str().unwrap().to_string();

    let row: (Vec<u8>,) = sqlx::query_as("SELECT secret_encrypted FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    let secret = crypto::totp::decrypt_totp_secret(&row.0, &harness.state.totp_key).unwrap();
    let totp = crypto::totp::create_totp_instance(&secret).unwrap();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    harness
        .send_request("POST", "/auth/mfa/totp/verify", Some(&token), Some(json!({ "code": totp.generate(now) })))
        .await;

    // 1st login with recovery code
    let (_, b_login1) = harness
        .send_request("POST", "/auth/login", None, Some(json!({ "email": email, "password": "Password123!" })))
        .await;
    let mfa_token1 = b_login1["mfa_token"].as_str().unwrap();

    let (s1, _) = harness
        .send_request("POST", "/auth/login/recovery", None, Some(json!({ "mfa_token": mfa_token1, "recovery_code": recovery_code })))
        .await;
    assert_eq!(s1, StatusCode::OK);

    // 2nd login with same recovery code (and fresh mfa_token)
    let (_, b_login2) = harness
        .send_request("POST", "/auth/login", None, Some(json!({ "email": email, "password": "Password123!" })))
        .await;
    let mfa_token2 = b_login2["mfa_token"].as_str().unwrap();

    let (s2, b2) = harness
        .send_request("POST", "/auth/login/recovery", None, Some(json!({ "mfa_token": mfa_token2, "recovery_code": recovery_code })))
        .await;
    assert_eq!(s2, StatusCode::UNAUTHORIZED);
    assert_eq!(b2["error"]["code"], "mfa_invalid");
}

#[tokio::test]
async fn test_login_recovery_wrong_code_returns_401() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, email) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(json!({ "current_password": "Password123!" })))
        .await;
    let row: (Vec<u8>,) = sqlx::query_as("SELECT secret_encrypted FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    let secret = crypto::totp::decrypt_totp_secret(&row.0, &harness.state.totp_key).unwrap();
    let totp = crypto::totp::create_totp_instance(&secret).unwrap();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    harness
        .send_request("POST", "/auth/mfa/totp/verify", Some(&token), Some(json!({ "code": totp.generate(now) })))
        .await;

    let (_, b_login) = harness
        .send_request("POST", "/auth/login", None, Some(json!({ "email": email, "password": "Password123!" })))
        .await;
    let mfa_token = b_login["mfa_token"].as_str().unwrap();

    let (status, body) = harness
        .send_request("POST", "/auth/login/recovery", None, Some(json!({ "mfa_token": mfa_token, "recovery_code": "WRONGCODE1" })))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "mfa_invalid");
}

#[tokio::test]
async fn test_login_recovery_missing_token_returns_401() {
    let harness = TestHarness::new().await;
    let (status, body) = harness
        .send_request("POST", "/auth/login/recovery", None, Some(json!({ "mfa_token": "bogus.token", "recovery_code": "ABCDEFGHIJ" })))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_token");
}

// ============================================================================
// 7. CROSS-CUTTING AUDIT & CONTRACT TESTS
// ============================================================================

#[tokio::test]
async fn test_cross_cutting_no_secrets_in_audit_log() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, token, email) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    // Enroll
    let (_, b_enr) = harness
        .send_request("POST", "/auth/mfa/totp/enroll", Some(&token), Some(json!({ "current_password": "Password123!" })))
        .await;
    let raw_secret = b_enr["secret"].as_str().unwrap();
    let recovery_code = b_enr["recovery_codes"][0].as_str().unwrap().to_string();

    // Verify
    let row: (Vec<u8>,) = sqlx::query_as("SELECT secret_encrypted FROM mfa_credentials WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_one(&harness.state.db_pool)
        .await
        .unwrap();
    let secret = crypto::totp::decrypt_totp_secret(&row.0, &harness.state.totp_key).unwrap();
    let totp = crypto::totp::create_totp_instance(&secret).unwrap();
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let totp_code = totp.generate(now);

    harness
        .send_request("POST", "/auth/mfa/totp/verify", Some(&token), Some(json!({ "code": totp_code })))
        .await;

    // Login Challenge + Recovery
    let (_, b_login) = harness
        .send_request("POST", "/auth/login", None, Some(json!({ "email": email, "password": "Password123!" })))
        .await;
    let mfa_token = b_login["mfa_token"].as_str().unwrap();

    harness
        .send_request("POST", "/auth/login/recovery", None, Some(json!({ "mfa_token": mfa_token, "recovery_code": recovery_code })))
        .await;

    // Fetch all audit rows for user
    let rows: Vec<(String, Value)> = sqlx::query_as("SELECT event, metadata FROM audit_log WHERE user_id = $1")
        .bind(user_id.into_inner())
        .fetch_all(&harness.state.db_pool)
        .await
        .unwrap();

    assert!(rows.iter().any(|r| r.0 == "auth.mfa.enrolled"));
    assert!(rows.iter().any(|r| r.0 == "auth.mfa.enrollment_confirmed"));
    assert!(rows.iter().any(|r| r.0 == "auth.mfa.recovery_used"));

    // Verify zero secrets leaked in metadata
    for (_, metadata) in rows {
        let meta_str = metadata.to_string();
        assert!(!meta_str.contains(raw_secret), "Secret leaked in audit log!");
        assert!(!meta_str.contains(&recovery_code), "Recovery code leaked in audit log!");
        assert!(!meta_str.contains("Password123!"), "Password leaked in audit log!");
        assert!(!meta_str.contains("$argon2id$"), "Hash leaked in audit log!");
    }
}

#[tokio::test]
async fn test_mfa_token_scope_rejected_by_auth_guard() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, _, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    let jti = Uuid::now_v7();
    let mfa_token = harness.issue_mfa_token(user_id, UserRole::Customer, 1, jti);

    // Presenting mfa_token to /auth/me should be rejected with 401 invalid_token
    let (status, body) = harness.send_request("GET", "/auth/me", Some(&mfa_token), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_token");
    assert_eq!(body["error"]["message"], "Token scope must be 'access'");
}

#[tokio::test]
async fn test_error_response_matches_phase3_shape() {
    let harness = TestHarness::new().await;
    let (status, body) = harness
        .send_request("POST", "/auth/login/totp", None, Some(json!({ "mfa_token": "bad", "code": "bad" })))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.get("error").is_some());
    assert!(body["error"].get("code").is_some());
    assert!(body["error"].get("message").is_some());
}

#[tokio::test]
async fn test_concurrent_recovery_code_use_only_one_succeeds() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, _, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    // Enroll MFA
    let (secret_bytes, _) = crypto::totp::generate_totp_secret();
    let secret_enc = crypto::totp::encrypt_totp_secret(&secret_bytes, &harness.state.totp_key).unwrap();
    let (plain_codes, hashes) = crypto::totp::generate_recovery_codes(10).unwrap();
    let mfa_id = Uuid::now_v7();

    sqlx::query(
        r#"
        INSERT INTO mfa_credentials (id, user_id, kind, secret_encrypted, recovery_codes_hash, confirmed_at, created_at)
        VALUES ($1, $2, 'totp', $3, $4, now(), now())
        "#,
    )
    .bind(mfa_id)
    .bind(user_id.into_inner())
    .bind(&secret_enc)
    .bind(&hashes)
    .execute(&harness.state.db_pool)
    .await
    .unwrap();

    // Issue two valid mfa_tokens for this user
    let now_ts = chrono::Utc::now().timestamp();
    let jti1 = Uuid::now_v7();
    let claims1 = Claims {
        sub: user_id.into_inner().to_string(),
        iss: DEFAULT_ISSUER.to_string(),
        aud: DEFAULT_AUDIENCE.to_string(),
        exp: now_ts + 300,
        iat: now_ts,
        nbf: now_ts,
        jti: jti1,
        sid: Uuid::nil(),
        auth_version: 1,
        role: "CUSTOMER".to_string(),
        scope: "mfa_pending".to_string(),
    };
    let token1 = harness.state.jwt_keys.sign_access_token(&claims1).unwrap();

    let jti2 = Uuid::now_v7();
    let claims2 = Claims {
        sub: user_id.into_inner().to_string(),
        iss: DEFAULT_ISSUER.to_string(),
        aud: DEFAULT_AUDIENCE.to_string(),
        exp: now_ts + 300,
        iat: now_ts,
        nbf: now_ts,
        jti: jti2,
        sid: Uuid::nil(),
        auth_version: 1,
        role: "CUSTOMER".to_string(),
        scope: "mfa_pending".to_string(),
    };
    let token2 = harness.state.jwt_keys.sign_access_token(&claims2).unwrap();

    // Store nonces in Redis
    let mut redis = harness.state.redis.clone().unwrap();
    let _: Result<(), _> = redis.set_ex(format!("mfa:challenge:{}", jti1), user_id.into_inner().to_string(), 300).await;
    let _: Result<(), _> = redis.set_ex(format!("mfa:challenge:{}", jti2), user_id.into_inner().to_string(), 300).await;

    let target_code = plain_codes[0].clone();

    // Spawn two concurrent requests with the SAME recovery code
    let req1 = {
        let t1 = token1.clone();
        let code = target_code.clone();
        harness.send_request(
            "POST",
            "/auth/login/recovery",
            None,
            Some(json!({ "mfa_token": t1, "recovery_code": code })),
        )
    };
    let req2 = {
        let t2 = token2.clone();
        let code = target_code.clone();
        harness.send_request(
            "POST",
            "/auth/login/recovery",
            None,
            Some(json!({ "mfa_token": t2, "recovery_code": code })),
        )
    };

    let ((status1, body1), (status2, body2)) = tokio::join!(req1, req2);

    let statuses = vec![status1, status2];
    assert!(statuses.contains(&StatusCode::OK), "Exactly one request must succeed (200)");
    assert!(statuses.contains(&StatusCode::UNAUTHORIZED), "Exactly one request must fail (401)");

    let failed_body = if status1 == StatusCode::UNAUTHORIZED { body1 } else { body2 };
    assert_eq!(failed_body["error"]["code"], "mfa_invalid");

    // Assert array length went from 10 -> 9
    let (remaining_hashes,): (Vec<String>,) = sqlx::query_as(
        "SELECT recovery_codes_hash FROM mfa_credentials WHERE user_id = $1",
    )
    .bind(user_id.into_inner())
    .fetch_one(&harness.state.db_pool)
    .await
    .unwrap();

    assert_eq!(remaining_hashes.len(), 9);
}

#[tokio::test]
async fn test_mfa_wrong_code_emits_failure_audit_events() {
    let harness = TestHarness::new().await;
    let (user_id, _, _, _, _, _, _) = harness
        .create_user_with_session(UserRole::Customer, UserStatus::Active, 1)
        .await;

    // Enroll MFA
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
    .bind(user_id.into_inner())
    .bind(&secret_enc)
    .bind(&hashes)
    .execute(&harness.state.db_pool)
    .await
    .unwrap();

    // 1. Wrong TOTP code
    let jti1 = Uuid::now_v7();
    let mfa_token1 = harness.issue_mfa_token(user_id, UserRole::Customer, 1, jti1);
    let mut redis = harness.state.redis.clone().unwrap();
    let _: Result<(), _> = redis.set_ex(format!("mfa:challenge:{}", jti1), user_id.into_inner().to_string(), 300).await;

    let (s_totp, b_totp) = harness
        .send_request("POST", "/auth/login/totp", None, Some(json!({ "mfa_token": mfa_token1, "code": "000000" })))
        .await;
    assert_eq!(s_totp, StatusCode::UNAUTHORIZED);
    assert_eq!(b_totp["error"]["code"], "mfa_invalid");

    // 2. Wrong Recovery code
    let jti2 = Uuid::now_v7();
    let mfa_token2 = harness.issue_mfa_token(user_id, UserRole::Customer, 1, jti2);
    let _: Result<(), _> = redis.set_ex(format!("mfa:challenge:{}", jti2), user_id.into_inner().to_string(), 300).await;

    let (s_rec, b_rec) = harness
        .send_request("POST", "/auth/login/recovery", None, Some(json!({ "mfa_token": mfa_token2, "recovery_code": "INVALIDRC1" })))
        .await;
    assert_eq!(s_rec, StatusCode::UNAUTHORIZED);
    assert_eq!(b_rec["error"]["code"], "mfa_invalid");

    // Query audit log
    let rows: Vec<(String, serde_json::Value)> = sqlx::query_as(
        "SELECT event, metadata FROM audit_log WHERE user_id = $1 ORDER BY created_at ASC",
    )
    .bind(user_id.into_inner())
    .fetch_all(&harness.state.db_pool)
    .await
    .unwrap();

    let totp_fail = rows.iter().find(|r| r.0 == "auth.mfa.login_failed").expect("auth.mfa.login_failed must be present");
    assert_eq!(totp_fail.1["method"], "totp");
    assert_eq!(totp_fail.1["user_id"], user_id.into_inner().to_string());

    let rec_fail = rows.iter().find(|r| r.0 == "auth.mfa.recovery_failed").expect("auth.mfa.recovery_failed must be present");
    assert_eq!(rec_fail.1["method"], "recovery");
    assert_eq!(rec_fail.1["user_id"], user_id.into_inner().to_string());

    // Verify absence of sensitive tokens/codes in metadata
    let totp_str = totp_fail.1.to_string();
    assert!(!totp_str.contains("000000"));
    assert!(!totp_str.contains(&mfa_token1));

    let rec_str = rec_fail.1.to_string();
    assert!(!rec_str.contains("INVALIDRC1"));
    assert!(!rec_str.contains(&mfa_token2));
}

