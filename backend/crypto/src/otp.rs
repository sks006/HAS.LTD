// backend/crypto/src/otp.rs

use rand::Rng;
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Generates a cryptographically secure 6-digit numeric OTP string (100000..=999999).
pub fn generate_otp_code() -> String {
    let num: u32 = rand::rngs::OsRng.gen_range(100_000..=999_999);
    format!("{:06}", num)
}

/// Computes the SHA-256 hash of (pepper || user_id_bytes || code_bytes) and returns lowercase hex.
/// Pre-seeded Decision 2: sha256(pepper || user_id_bytes || code_bytes)
pub fn hash_otp(pepper: &[u8; 32], user_id: Uuid, code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pepper);
    hasher.update(user_id.as_bytes());
    hasher.update(code.as_bytes());
    hex::encode(hasher.finalize())
}

/// Verifies a candidate OTP code against an expected hex hash using constant-time equality check.
pub fn verify_otp_hash(pepper: &[u8; 32], user_id: Uuid, code: &str, expected: &str) -> bool {
    let computed = hash_otp(pepper, user_id, code);
    crate::token::constant_time_eq(computed.as_bytes(), expected.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_otp_code_length_and_range() {
        for _ in 0..100 {
            let code = generate_otp_code();
            assert_eq!(code.len(), 6, "Code must be exactly 6 characters");
            let parsed: u32 = code.parse().expect("Code must be numeric");
            assert!(parsed >= 100_000 && parsed <= 999_999);
        }
    }

    #[test]
    fn test_otp_hash_round_trip() {
        let pepper = [42u8; 32];
        let user_id = Uuid::new_v4();
        let code = generate_otp_code();

        let expected_hash = hash_otp(&pepper, user_id, &code);
        assert_eq!(expected_hash.len(), 64, "SHA-256 hex must be 64 characters");

        assert!(verify_otp_hash(&pepper, user_id, &code, &expected_hash));
    }

    #[test]
    fn test_otp_hash_wrong_code_fails() {
        let pepper = [42u8; 32];
        let user_id = Uuid::new_v4();
        let code = "123456";
        let wrong_code = "654321";

        let hash = hash_otp(&pepper, user_id, code);
        assert!(!verify_otp_hash(&pepper, user_id, wrong_code, &hash));
    }

    #[test]
    fn test_otp_hash_same_code_different_user_fails() {
        let pepper = [42u8; 32];
        let user_a = Uuid::new_v4();
        let user_b = Uuid::new_v4();
        let code = "123456";

        let hash_a = hash_otp(&pepper, user_a, code);
        assert!(!verify_otp_hash(&pepper, user_b, code, &hash_a));
    }

    #[test]
    fn test_otp_hash_different_pepper_fails() {
        let pepper_a = [42u8; 32];
        let pepper_b = [77u8; 32];
        let user_id = Uuid::new_v4();
        let code = "123456";

        let hash_a = hash_otp(&pepper_a, user_id, code);
        assert!(!verify_otp_hash(&pepper_b, user_id, code, &hash_a));
    }
}
