use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
}

#[derive(Clone)]
pub struct AppConfig {
    pub app_name: String,
    pub environment: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_name: "HAS.LTD".to_string(),
            environment: "development".to_string(),
        }
    }
}
