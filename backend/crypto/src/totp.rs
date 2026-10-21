// backend/crypto/src/totp.rs

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::rngs::OsRng;
use rand::RngCore;
use totp_rs::{Algorithm, TOTP};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::errors::CryptoError;
// Unambiguous alphanumeric alphabet: 32 characters (excludes 0, O, 1, I, l)
const RECOVERY_ALPHABET: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
const RECOVERY_CODE_LEN: usize = 10;
const AES_GCM_NONCE_LEN: usize = 12;
const AES_GCM_TAG_LEN: usize = 16;

/// Encrypted TOTP secret wrapper that zeroizes its inner buffer on drop.
#[derive(Debug, Clone, PartialEq, Eq, Zeroize, ZeroizeOnDrop)]
pub struct TotpSecret(pub Vec<u8>);

impl TotpSecret {
    pub fn new(secret: Vec<u8>) -> Self {
        Self(secret)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

/// Helper to construct a standard RFC 6238 TOTP instance with HMAC-SHA1, 6 digits, 30s period,
/// and skew tolerance of 1 step (tolerating -1, 0, +1 steps).
pub fn create_totp_instance(secret: &[u8]) -> Result<TOTP, CryptoError> {
    TOTP::new(
        Algorithm::SHA1,
        6,
        1,  // Skew window: 1 step tolerance (-30s, +30s)
        30, // 30-second period
        secret.to_vec(),
        Some("HAS.LTD".to_string()),
        "user@has.ltd".to_string(),
    )
    .map_err(|e| CryptoError::TotpError(e.to_string()))
}

/// Generates a new 160-bit (20-byte) cryptographically secure TOTP secret and returns:
/// - `plaintext_secret`: 20 raw bytes from `OsRng`
/// - `otpauth_url`: standard `otpauth://totp/...` URI for QR code generation
pub fn generate_totp_secret() -> (Vec<u8>, String) {
    let mut secret = vec![0u8; 20];
    OsRng.fill_bytes(&mut secret);

    let totp = create_totp_instance(&secret)
        .expect("static TOTP parameters are always valid");
    let url = totp.get_url();

    (secret, url)
}

/// Verifies a 6-digit TOTP code against the secret at a specific Unix timestamp in seconds.
///
/// Accepts codes within the skew window $[-1, 0, +1]$ steps ($[-30\text{s}, +30\text{s}]$).
/// Rejects codes at $\le -2$ or $\ge +2$ steps.
pub fn verify_totp_at_time(secret: &[u8], code: &str, timestamp: u64) -> Result<bool, CryptoError> {
    let totp = create_totp_instance(secret)?;
    Ok(totp.check(code, timestamp))
}

/// Verifies a 6-digit TOTP code against the secret at the current system time.
pub fn verify_totp(secret: &[u8], code: &str) -> Result<bool, CryptoError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| CryptoError::TotpError(e.to_string()))?
        .as_secs();

    verify_totp_at_time(secret, code, now)
}

/// Generates `count` recovery codes and their Argon2id PHC hashes.
///
/// - Each code is 10 characters long, chosen from an unambiguous 32-character alphabet
///   (no `0`, `O`, `1`, `I`, `l`).
/// - Returns `(plaintext_codes, hash_codes)`:
///   - `plaintext_codes`: Shown to the user exactly once during MFA enrollment.
///   - `hash_codes`: Argon2id PHC strings persisted in the database.
pub fn generate_recovery_codes(count: usize) -> Result<(Vec<String>, Vec<String>), CryptoError> {
    let mut plaintext_codes = Vec::with_capacity(count);
    let mut hash_codes = Vec::with_capacity(count);

    for _ in 0..count {
        let mut code_bytes = [0u8; RECOVERY_CODE_LEN];
        let mut random_bytes = [0u8; RECOVERY_CODE_LEN];
        OsRng.fill_bytes(&mut random_bytes);

        for i in 0..RECOVERY_CODE_LEN {
            let idx = (random_bytes[i] as usize) % RECOVERY_ALPHABET.len();
            code_bytes[i] = RECOVERY_ALPHABET[idx];
        }

        let code_str = String::from_utf8(code_bytes.to_vec())
            .expect("recovery alphabet contains only ASCII characters");
        let hash_str = crate::password::hash_password(&code_str)?;

        plaintext_codes.push(code_str);
        hash_codes.push(hash_str);
    }

    Ok((plaintext_codes, hash_codes))
}

/// Verifies a recovery code against the stored Argon2id hashes and returns
/// the matching index so the caller can delete or mark the single-use recovery code.
pub fn verify_recovery_code_index(code: &str, hashes: &[String]) -> Result<Option<usize>, CryptoError> {
    let normalized = code.trim().to_uppercase();

    for (idx, stored_hash) in hashes.iter().enumerate() {
        if crate::password::verify_password(&normalized, stored_hash)? {
            return Ok(Some(idx));
        }
    }

    Ok(None)
}

/// Verifies a recovery code against the stored Argon2id hashes.
pub fn verify_recovery_code(code: &str, hashes: &[String]) -> Result<bool, CryptoError> {
    verify_recovery_code_index(code, hashes).map(|opt| opt.is_some())
}

/// Encrypts plaintext TOTP secret bytes using AES-256-GCM authenticated encryption.
///
/// Prepends a randomly generated 12-byte nonce to the ciphertext:
/// `[12-byte nonce][ciphertext_and_16_byte_auth_tag]`
pub fn encrypt_totp_secret(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| CryptoError::InvalidKeyLength { expected: 32, actual: key.len() })?;

    let mut nonce_bytes = [0u8; AES_GCM_NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|_| CryptoError::AeadEncryptionFailed)?;

    let mut output = Vec::with_capacity(AES_GCM_NONCE_LEN + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

/// Decrypts ciphertext TOTP secret bytes using AES-256-GCM.
///
/// Expects format: `[12-byte nonce][ciphertext_and_16_byte_auth_tag]`
pub fn decrypt_totp_secret(ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
    let min_len = AES_GCM_NONCE_LEN + AES_GCM_TAG_LEN;
    if ciphertext.len() < min_len {
        return Err(CryptoError::AeadDecryptionFailed);
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| CryptoError::InvalidKeyLength { expected: 32, actual: key.len() })?;

    let (nonce_bytes, encrypted_payload) = ciphertext.split_at(AES_GCM_NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    let decrypted = cipher.decrypt(nonce, encrypted_payload)
        .map_err(|_| CryptoError::AeadDecryptionFailed)?;

    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_roundtrip_fixed_secret_and_time() {
        let secret = b"12345678901234567890"; // 20 bytes standard
        let timestamp = 1700000000u64;

        let totp = create_totp_instance(secret).expect("totp instance should be created");
        let code = totp.generate(timestamp);
        assert_eq!(code.len(), 6, "TOTP code must be 6 digits");

        let is_valid = verify_totp_at_time(secret, &code, timestamp)
            .expect("verification should succeed");
        assert!(is_valid, "Valid code generated at timestamp must verify to true");
    }

    #[test]
    fn test_totp_skew_window_tolerance() {
        let secret = b"12345678901234567890";
        let t = 1700000000u64;
        let totp = create_totp_instance(secret).unwrap();

        let code_step_0 = totp.generate(t);
        let code_step_minus_1 = totp.generate(t - 30);
        let code_step_plus_1 = totp.generate(t + 30);
        let code_step_minus_2 = totp.generate(t - 60);
        let code_step_plus_2 = totp.generate(t + 60);

        // Current step 0 must be accepted
        assert!(verify_totp_at_time(secret, &code_step_0, t).unwrap(), "Step 0 must be accepted");
        // Skew window -1 (-30s) must be accepted
        assert!(verify_totp_at_time(secret, &code_step_minus_1, t).unwrap(), "Step -1 must be accepted");
        // Skew window +1 (+30s) must be accepted
        assert!(verify_totp_at_time(secret, &code_step_plus_1, t).unwrap(), "Step +1 must be accepted");

        // Skew window -2 (-60s) must be rejected
        assert!(!verify_totp_at_time(secret, &code_step_minus_2, t).unwrap(), "Step -2 must be rejected");
        // Skew window +2 (+60s) must be rejected
        assert!(!verify_totp_at_time(secret, &code_step_plus_2, t).unwrap(), "Step +2 must be rejected");
    }

    #[test]
    fn test_totp_wrong_code_rejected() {
        let secret = b"12345678901234567890";
        let timestamp = 1700000000u64;
        let totp = create_totp_instance(secret).unwrap();
        let valid_code = totp.generate(timestamp);

        // Construct invalid code
        let invalid_code = if valid_code == "123456" { "654321" } else { "123456" };
        let is_valid = verify_totp_at_time(secret, invalid_code, timestamp).unwrap();
        assert!(!is_valid, "Invalid code must be rejected");
    }

    #[test]
    fn test_recovery_codes_roundtrip() {
        let count = 3;
        let (plaintext_codes, hash_codes) = generate_recovery_codes(count).expect("recovery code generation must succeed");

        assert_eq!(plaintext_codes.len(), count);
        assert_eq!(hash_codes.len(), count);

        for (i, code) in plaintext_codes.iter().enumerate() {
            assert_eq!(code.len(), 10, "Recovery code must be 10 characters");

            // [FIX-H5-TEST]: Assert stored hash is an Argon2id PHC string
            assert!(
                hash_codes[i].starts_with("$argon2id$"),
                "Stored recovery code hash must be an Argon2id PHC string, got: {}",
                hash_codes[i]
            );

            // Verify unambiguous alphabet
            for ch in code.chars() {
                assert!(RECOVERY_ALPHABET.contains(&(ch as u8)), "Disallowed char in recovery code: {}", ch);
            }

            // Verify matching with Argon2id
            let is_valid = verify_recovery_code(code, &hash_codes).unwrap();
            assert!(is_valid);

            let matched_idx = verify_recovery_code_index(code, &hash_codes).unwrap();
            assert_eq!(matched_idx, Some(i));
        }

        // Non-existent recovery code
        let is_valid = verify_recovery_code("XXXXXXXXXX", &hash_codes).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_reused_recovery_code_rejected_at_application_layer() {
        let (plaintext_codes, mut hash_codes) = generate_recovery_codes(2).expect("generation must succeed");
        let first_code = &plaintext_codes[0];

        // First use: successful match and removal of index
        let matched_idx = verify_recovery_code_index(first_code, &hash_codes)
            .expect("verification should succeed");
        assert_eq!(matched_idx, Some(0));

        // Simulate single-use consumption at application/database layer
        let idx = matched_idx.unwrap();
        hash_codes.remove(idx);

        // Second use (re-use attack): must be rejected
        let second_attempt = verify_recovery_code(first_code, &hash_codes)
            .expect("verification should succeed");
        assert!(!second_attempt, "Re-used recovery code must be rejected once consumed");
    }

    #[test]
    fn test_aes_gcm_totp_secret_encryption_roundtrip() {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);

        let plaintext_secret = b"my_precious_totp_seed_bytes_1234567890";

        // Encrypt
        let ciphertext = encrypt_totp_secret(plaintext_secret, &key).expect("encryption must succeed");
        assert!(ciphertext.len() > 12 + 16, "Ciphertext must contain 12-byte nonce + payload + tag");

        // Decrypt
        let decrypted = decrypt_totp_secret(&ciphertext, &key).expect("decryption must succeed");
        assert_eq!(&decrypted[..], plaintext_secret);

        // Tamper test: modify one byte in payload
        let mut tampered = ciphertext.clone();
        let last = tampered.len() - 1;
        tampered[last] ^= 0x01;

        let res = decrypt_totp_secret(&tampered, &key);
        assert!(res.is_err(), "Tampered ciphertext must fail authentication tag verification");

        // Wrong key test
        let mut wrong_key = [0u8; 32];
        OsRng.fill_bytes(&mut wrong_key);
        let res_wrong_key = decrypt_totp_secret(&ciphertext, &wrong_key);
        assert!(res_wrong_key.is_err(), "Decryption with wrong key must fail");
    }
}
