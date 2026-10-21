// backend/crypto/src/token.rs

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use zeroize::Zeroize;

/// Computes the SHA-256 digest of the given raw byte slice and formats it as a 64-character lowercase hex string.
pub fn sha256_hex(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let digest = hasher.finalize();
    hex::encode(digest)
}

/// Generates a cryptographically secure refresh token pair.
///
/// Returns `(plaintext_base64url, sha256_hex)`:
/// - `plaintext_base64url`: 32 random bytes from `OsRng`, unpadded URL-safe Base64 encoded (43 chars).
///   Sent to the client and never stored in the database.
/// - `sha256_hex`: SHA-256 hex digest computed directly over the raw 32 bytes (NOT the Base64 representation).
///   Stored in the database `refresh_tokens.token_hash` column.
pub fn generate_refresh_token() -> (String, String) {
    let mut raw_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut raw_bytes);

    let plaintext_base64url = URL_SAFE_NO_PAD.encode(&raw_bytes);
    let sha256_hex_digest = sha256_hex(&raw_bytes);

    raw_bytes.zeroize();
    (plaintext_base64url, sha256_hex_digest)
}

/// Performs a constant-time equality check over two byte slices to prevent timing side channels.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

/// Generates a cryptographically secure opaque token (e.g., for email verification,
/// password reset tokens, or cryptographic salt) of the requested byte length, encoded as unpadded URL-safe Base64.
pub fn generate_opaque_token(bytes: usize) -> String {
    let mut raw = vec![0u8; bytes];
    OsRng.fill_bytes(&mut raw);
    let encoded = URL_SAFE_NO_PAD.encode(&raw);
    raw.zeroize();
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hex_known_vector() {
        // NIST SHA-256 standard test vector for "abc"
        let input = b"abc";
        let expected = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert_eq!(sha256_hex(input), expected);
    }

    #[test]
    fn test_generate_refresh_token_lengths_and_hashing() {
        let (plaintext, hash) = generate_refresh_token();

        // 32 bytes encoded in base64url unpadded is ceil(32 * 4 / 3) = 43 chars
        assert_eq!(plaintext.len(), 43, "Refresh token plaintext must be exactly 43 characters");
        // SHA-256 hex digest is 32 bytes * 2 = 64 characters
        assert_eq!(hash.len(), 64, "Token hash must be exactly 64 hex characters");

        // Independent generation produces distinct plaintext and distinct hash
        let (plaintext2, hash2) = generate_refresh_token();
        assert_ne!(plaintext, plaintext2);
        assert_ne!(hash, hash2);

        // Verify that hashing the raw bytes decoded from plaintext matches the returned hash
        let decoded = URL_SAFE_NO_PAD.decode(plaintext.as_bytes()).expect("must be valid base64url");
        assert_eq!(decoded.len(), 32);
        assert_eq!(sha256_hex(&decoded), hash, "Hash must be computed over raw bytes, not base64 string");
    }

    #[test]
    fn test_constant_time_eq_correctness() {
        assert!(constant_time_eq(b"identical_secret_bytes", b"identical_secret_bytes"));
        assert!(!constant_time_eq(b"identical_secret_bytes", b"different_secret_bytes"));
        assert!(!constant_time_eq(b"short", b"longer_string"));
        assert!(!constant_time_eq(b"", b"non_empty"));
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn test_generate_opaque_token() {
        let token16 = generate_opaque_token(16);
        let token32 = generate_opaque_token(32);

        let decoded16 = URL_SAFE_NO_PAD.decode(token16.as_bytes()).unwrap();
        let decoded32 = URL_SAFE_NO_PAD.decode(token32.as_bytes()).unwrap();

        assert_eq!(decoded16.len(), 16);
        assert_eq!(decoded32.len(), 32);
    }
}
