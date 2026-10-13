use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    ReadCatalog,
    WriteCatalog,
    FulfillOrder,
    ManageUsers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleClaims {
    pub permissions: Vec<Permission>,
}
