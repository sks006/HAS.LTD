use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AdjustInventoryDto {
    pub quantity_change: i32,
    pub reason: String,
}
