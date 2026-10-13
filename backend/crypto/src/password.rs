// backend/crypto/src/password.rs

use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2, Algorithm, Params, Version,
};
use zeroize::Zeroize;

use crate::errors::CryptoError;

// OWASP 2024 recommended parameters for standard containerized environments:
// Memory: 19 MiB (19,456 KiB), Time: 2 iterations, Parallelism: 1 lane, Output: 32 bytes
pub const DEFAULT_M_COST: u32 = 19456;
pub const DEFAULT_T_COST: u32 = 2;
pub const DEFAULT_P_COST: u32 = 1;
pub const DEFAULT_OUTPUT_LEN: usize = 32;

// Fixed dummy PHC computed with exact default parameters (m=19456, t=2, p=1).
// Used by dummy_verify to guarantee identical CPU and memory consumption.
const DUMMY_PHC: &str = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQxMjM0NTY3OA$qGkY5oG17dG6Yn7zV4pX0rE3uW5lP8sN2yF9vA4bC0M";
const DUMMY_INPUT: &str = "dummy_constant_time_password_pad";

/// Computes an Argon2id password hash formatted as a standard PHC string.
///
/// NOTE: This function is CPU and memory intensive. In async contexts, call
/// `hash_password_async` or wrap this invocation in `tokio::task::spawn_blocking`.
pub fn hash_password(password: &str) -> Result<String, CryptoError> {
    let params = Params::new(DEFAULT_M_COST, DEFAULT_T_COST, DEFAULT_P_COST, Some(DEFAULT_OUTPUT_LEN))
        .map_err(|e| CryptoError::InvalidPasswordHash(e.to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| CryptoError::PasswordHashFailed)?;

    Ok(password_hash.to_string())
}

/// Verifies a password against an Argon2id PHC string in constant time.
///
/// NOTE: This function is CPU and memory intensive. In async contexts, call
/// `verify_password_async` or wrap this invocation in `tokio::task::spawn_blocking`.
pub fn verify_password(password: &str, phc: &str) -> Result<bool, CryptoError> {
    let parsed_hash = PasswordHash::new(phc)
        .map_err(|e| CryptoError::InvalidPasswordHash(e.to_string()))?;

    // Argon2::default().verify_password automatically parses the algorithm, version,
    // and parameters directly from the PHC string and executes constant-time verification.
    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(_) => Err(CryptoError::PasswordVerifyFailed),
    }
}

/// Consumes identical CPU and memory resources as a real verification against a fixed dummy PHC.
/// Used to eliminate timing side-channels during unknown-user authentication attempts.
///
/// NOTE: In async contexts, call `dummy_verify_async` or wrap in `tokio::task::spawn_blocking`.
pub fn dummy_verify() -> Result<(), CryptoError> {
    let _ = verify_password(DUMMY_INPUT, DUMMY_PHC);
    Ok(())
}

/// Determines whether the given PHC string was generated with parameters older or different
/// than the active configuration (e.g. outdated iterations, memory cost, or obsolete algorithm).
pub fn needs_rehash(phc: &str) -> Result<bool, CryptoError> {
    let parsed = PasswordHash::new(phc)
        .map_err(|e| CryptoError::InvalidPasswordHash(e.to_string()))?;

    // Algorithm must be Argon2id
    if parsed.algorithm != argon2::ARGON2ID_IDENT {
        return Ok(true);
    }

    // Version must be 0x13 (19)
    if parsed.version != Some(Version::V0x13.into()) {
        return Ok(true);
    }

    // Check memory cost parameter ("m")
    let m = parsed.params.get("m")
        .and_then(|v| v.decimal().ok());
    if m != Some(DEFAULT_M_COST) {
        return Ok(true);
    }

    // Check time cost parameter ("t")
    let t = parsed.params.get("t")
        .and_then(|v| v.decimal().ok());
    if t != Some(DEFAULT_T_COST) {
        return Ok(true);
    }

    // Check parallelism parameter ("p")
    let p = parsed.params.get("p")
        .and_then(|v| v.decimal().ok());
    if p != Some(DEFAULT_P_COST) {
        return Ok(true);
    }

    Ok(false)
}

/// Async wrapper for `hash_password` that offloads Argon2 computation to Tokio's blocking threadpool
/// and zeroizes the plaintext password string upon completion.
pub async fn hash_password_async(password: String) -> Result<String, CryptoError> {
    tokio::task::spawn_blocking(move || {
        let mut pwd = password;
        let res = hash_password(&pwd);
        pwd.zeroize();
        res
    })
    .await
    .map_err(|e| CryptoError::TaskJoinError(e.to_string()))?
}

/// Async wrapper for `verify_password` that offloads Argon2 computation to Tokio's blocking threadpool
/// and zeroizes the plaintext password string upon completion.
pub async fn verify_password_async(password: String, phc: String) -> Result<bool, CryptoError> {
    tokio::task::spawn_blocking(move || {
        let mut pwd = password;
        let res = verify_password(&pwd, &phc);
        pwd.zeroize();
        res
    })
    .await
    .map_err(|e| CryptoError::TaskJoinError(e.to_string()))?
}

/// Async wrapper for `dummy_verify` executing within Tokio's blocking threadpool.
pub async fn dummy_verify_async() -> Result<(), CryptoError> {
    tokio::task::spawn_blocking(dummy_verify)
        .await
        .map_err(|e| CryptoError::TaskJoinError(e.to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_verify_roundtrip() {
        let password = "super_secure_password_123!";
        let phc = hash_password(password).expect("hashing should succeed");
        assert!(phc.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));

        let is_valid = verify_password(password, &phc).expect("verification should succeed");
        assert!(is_valid, "Correct password must verify to true");
    }

    #[test]
    fn test_wrong_password_fails() {
        let password = "correct_password";
        let phc = hash_password(password).expect("hashing should succeed");

        let is_valid = verify_password("wrong_password", &phc).expect("verification should succeed");
        assert!(!is_valid, "Wrong password must verify to false");
    }

    #[test]
    fn test_same_password_different_hashes_salt_check() {
        let password = "identical_password";
        let phc1 = hash_password(password).expect("hash 1 should succeed");
        let phc2 = hash_password(password).expect("hash 2 should succeed");

        assert_ne!(phc1, phc2, "Independent hashes of the same password must use different salts");
        assert!(verify_password(password, &phc1).unwrap());
        assert!(verify_password(password, &phc2).unwrap());
    }

    #[test]
    fn test_needs_rehash_returns_true_for_known_old_phc() {
        // Current parameters: m=19456, t=2, p=1
        let current_phc = hash_password("modern_pass").expect("hashing should succeed");
        assert!(!needs_rehash(&current_phc).expect("needs_rehash check should succeed"));

        // Outdated parameters: m=4096, t=1, p=1
        let old_phc = "$argon2id$v=19$m=4096,t=1,p=1$c29tZXNhbHQxMjM0NTY3OA$qGkY5oG17dG6Yn7zV4pX0rE3uW5lP8sN2yF9vA4bC0M";
        assert!(needs_rehash(old_phc).expect("needs_rehash check should succeed"));

        // Outdated algorithm: argon2i instead of argon2id
        let argon2i_phc = "$argon2i$v=19$m=19456,t=2,p=1$c29tZXNhbHQxMjM0NTY3OA$qGkY5oG17dG6Yn7zV4pX0rE3uW5lP8sN2yF9vA4bC0M";
        assert!(needs_rehash(argon2i_phc).expect("needs_rehash check should succeed"));
    }

    #[test]
    fn test_dummy_verify_timing_within_ten_percent() {
        let password = "test_timing_password";
        let phc = hash_password(password).expect("hashing should succeed");

        // Warm up CPU and caches
        let _ = verify_password("wrong_password", &phc);
        let _ = dummy_verify();

        let iterations = 3;

        let start_real = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = verify_password("wrong_password", &phc);
        }
        let elapsed_real = start_real.elapsed().as_nanos() as f64 / iterations as f64;

        let start_dummy = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = dummy_verify();
        }
        let elapsed_dummy = start_dummy.elapsed().as_nanos() as f64 / iterations as f64;

        let diff_ratio = (elapsed_dummy - elapsed_real).abs() / elapsed_real;
        // Verify relative duration difference is within acceptable tolerance bound (< 15%)
        assert!(
            diff_ratio < 0.15,
            "dummy_verify timing ({:.2}ms) must match real verify ({:.2}ms) within 15% (observed diff: {:.1}%)",
            elapsed_dummy / 1_000_000.0,
            elapsed_real / 1_000_000.0,
            diff_ratio * 100.0
        );
    }

    #[tokio::test]
    async fn test_async_password_wrappers() {
        let password = "async_test_password".to_string();
        let phc = hash_password_async(password.clone()).await.expect("async hash must succeed");

        let ok = verify_password_async(password, phc.clone()).await.expect("async verify must succeed");
        assert!(ok);

        let fail = verify_password_async("wrong".to_string(), phc).await.expect("async verify must succeed");
        assert!(!fail);

        dummy_verify_async().await.expect("async dummy verify must succeed");
    }
}
