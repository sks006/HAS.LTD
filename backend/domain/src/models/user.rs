// backend/domain/src/models/user.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Customer,
    Admin,
    Moderator, // only for store manager 
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    
    // Passwords must NEVER be deserialized from the database into the 
    // root User entity that gets serialized to the frontend.
    // We omit it here intentionally. Authentication requires a dedicated struct.
    
    pub name: String,
    pub role: UserRole, 
    
    pub is_active: bool, // Required to suspend malicious actors
    // Tracks the exact moment the cryptographically secure token was validated
    // User-requested closure or GDPR wipe.
    pub deleted_at: Option<DateTime<Utc>>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}



#[derive(Debug, Clone)]
pub struct UserIdentity {
    pub user: User,
    pub password_hash: String, // Stored as argon2id hash string
}