pub struct TokenProvider {
    pub secret: String,
}

impl TokenProvider {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn generate_token(&self, email: &str, role: &str) -> Result<String, String> {
        // Simple token generation placeholder
        Ok(format!("{}.{}.mockjwt", email, role))
    }
}
