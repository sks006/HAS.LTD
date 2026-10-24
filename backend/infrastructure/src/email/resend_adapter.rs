// backend/infrastructure/src/email/resend_adapter.rs

use std::time::Duration;
use async_trait::async_trait;
use domain::ports::email_port::{EmailError, EmailMessage, EmailPort};
use serde_json::json;

pub struct ResendEmailAdapter {
    client: reqwest::Client,
    api_key: String,
    from_email: String,
    base_url: String,
}

impl ResendEmailAdapter {
    pub fn new(api_key: impl Into<String>, from_email: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to build reqwest client for ResendEmailAdapter");

        Self {
            client,
            api_key: api_key.into(),
            from_email: from_email.into(),
            base_url: "https://api.resend.com".to_string(),
        }
    }

    pub fn with_base_url(
        api_key: impl Into<String>,
        from_email: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to build reqwest client for ResendEmailAdapter");

        Self {
            client,
            api_key: api_key.into(),
            from_email: from_email.into(),
            base_url: base_url.into(),
        }
    }

    // SAFETY: The `code` variable is strictly a 6-digit numeric string generated via OsRng
    // in crypto::otp::generate_otp_code, containing only ASCII digits [0-9]. It is therefore
    // safe from HTML injection and XSS attacks. Any future templates incorporating arbitrary
    // user-supplied text (e.g. names, notes, custom subjects) MUST use an HTML escaping utility
    // before inserting into HTML markup.
    fn render_html(template_id: &str, variables: &serde_json::Value) -> String {
        let code = variables.get("code").and_then(|v| v.as_str()).unwrap_or("");
        match template_id {
            "email_verify_otp" => format!(
                "<h2>Welcome to HAS.LTD</h2><p>Your email verification code is: <strong>{}</strong></p><p>This code will expire in 10 minutes.</p>",
                code
            ),
            "password_reset_otp" => format!(
                "<h2>HAS.LTD Password Reset</h2><p>Your password reset code is: <strong>{}</strong></p><p>This code will expire in 10 minutes. If you did not request this, please ignore this email.</p>",
                code
            ),
            _ => format!(
                "<p>HAS.LTD Notification</p><pre>{}</pre>",
                serde_json::to_string_pretty(variables).unwrap_or_default()
            ),
        }
    }
}

#[async_trait]
impl EmailPort for ResendEmailAdapter {
    async fn send(&self, msg: EmailMessage) -> Result<(), EmailError> {
        let html_content = Self::render_html(&msg.template_id, &msg.variables);
        let endpoint = format!("{}/emails", self.base_url.trim_end_matches('/'));

        let payload = json!({
            "from": self.from_email,
            "to": [msg.to],
            "subject": msg.subject,
            "html": html_content
        });

        let backoffs = [
            Duration::from_millis(250),
            Duration::from_millis(500),
            Duration::from_millis(1000),
        ];

        let mut last_error: Option<EmailError> = None;

        for attempt in 0..3 {
            let res = self
                .client
                .post(&endpoint)
                .bearer_auth(&self.api_key)
                .json(&payload)
                .send()
                .await;

            match res {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        return Ok(());
                    } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
                        let err_msg = format!("Resend returned HTTP {}", status.as_u16());
                        tracing::warn!(
                            attempt = attempt + 1,
                            status = status.as_u16(),
                            "Resend API returned retryable status"
                        );
                        last_error = Some(EmailError::Transient(err_msg));
                        if attempt < 2 {
                            tokio::time::sleep(backoffs[attempt]).await;
                            continue;
                        }
                    } else if status.is_client_error() {
                        let body = response.text().await.unwrap_or_default();
                        tracing::error!(
                            status = status.as_u16(),
                            body = %body,
                            "Resend API returned permanent client error"
                        );
                        return Err(EmailError::Permanent(format!(
                            "Resend HTTP {}: {}",
                            status.as_u16(),
                            body
                        )));
                    } else {
                        return Err(EmailError::Permanent(format!(
                            "Unexpected Resend HTTP status: {}",
                            status.as_u16()
                        )));
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        attempt = attempt + 1,
                        error = %e,
                        "Resend request network or timeout error"
                    );
                    last_error = Some(EmailError::Transient(format!("Network/timeout error: {}", e)));
                    if attempt < 1 {
                        tokio::time::sleep(backoffs[attempt]).await;
                        continue;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            EmailError::Transient("Resend dispatch failed after 3 attempts".to_string())
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_resend_adapter_success_200() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/emails"))
            .and(header("authorization", "Bearer test_api_key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "msg_123"})))
            .expect(1)
            .mount(&mock_server)
            .await;

        let adapter = ResendEmailAdapter::with_base_url(
            "test_api_key",
            "noreply@has.ltd",
            mock_server.uri(),
        );

        let msg = EmailMessage {
            to: "user@example.com".to_string(),
            subject: "Verify your email".to_string(),
            template_id: "email_verify_otp".to_string(),
            variables: json!({"code": "123456"}),
        };

        let result = adapter.send(msg).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_resend_adapter_permanent_error_400() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/emails"))
            .respond_with(ResponseTemplate::new(400).set_body_json(json!({"message": "Invalid email"})))
            .expect(1)
            .mount(&mock_server)
            .await;

        let adapter = ResendEmailAdapter::with_base_url(
            "test_api_key",
            "noreply@has.ltd",
            mock_server.uri(),
        );

        let msg = EmailMessage {
            to: "invalid-email".to_string(),
            subject: "Verify".to_string(),
            template_id: "email_verify_otp".to_string(),
            variables: json!({"code": "123456"}),
        };

        let result = adapter.send(msg).await;
        match result {
            Err(EmailError::Permanent(msg)) => assert!(msg.contains("400")),
            other => panic!("Expected Permanent error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_resend_adapter_retry_on_429_then_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/emails"))
            .respond_with(ResponseTemplate::new(429))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/emails"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "msg_429_success"})))
            .mount(&mock_server)
            .await;

        let adapter = ResendEmailAdapter::with_base_url(
            "test_api_key",
            "noreply@has.ltd",
            mock_server.uri(),
        );

        let msg = EmailMessage {
            to: "user@example.com".to_string(),
            subject: "Retry test".to_string(),
            template_id: "email_verify_otp".to_string(),
            variables: json!({"code": "654321"}),
        };

        let result = adapter.send(msg).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_resend_adapter_retry_on_500_then_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/emails"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/emails"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "msg_500_success"})))
            .mount(&mock_server)
            .await;

        let adapter = ResendEmailAdapter::with_base_url(
            "test_api_key",
            "noreply@has.ltd",
            mock_server.uri(),
        );

        let msg = EmailMessage {
            to: "user@example.com".to_string(),
            subject: "Retry test".to_string(),
            template_id: "email_verify_otp".to_string(),
            variables: json!({"code": "654321"}),
        };

        let result = adapter.send(msg).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_resend_adapter_persistent_500_fails_transient() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/emails"))
            .respond_with(ResponseTemplate::new(500))
            .expect(3)
            .mount(&mock_server)
            .await;

        let adapter = ResendEmailAdapter::with_base_url(
            "test_api_key",
            "noreply@has.ltd",
            mock_server.uri(),
        );

        let msg = EmailMessage {
            to: "user@example.com".to_string(),
            subject: "Persistent 500".to_string(),
            template_id: "email_verify_otp".to_string(),
            variables: json!({"code": "111111"}),
        };

        let result = adapter.send(msg).await;
        match result {
            Err(EmailError::Transient(msg)) => assert!(msg.contains("500")),
            other => panic!("Expected Transient error, got {:?}", other),
        }
    }
}
