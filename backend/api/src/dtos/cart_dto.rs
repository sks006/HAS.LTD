use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItemMutationDto {
    pub variant_id: Uuid,
    pub quantity_change: i32,
}
