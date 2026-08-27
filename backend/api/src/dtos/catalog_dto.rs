use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductQueryDto {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub category: Option<String>,
    pub search: Option<String>,
}
