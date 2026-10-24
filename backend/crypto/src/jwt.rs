// backend/crypto/src/jwt.rs

use std::collections::HashMap;
use std::path::Path;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::CryptoError;

pub const DEFAULT_ISSUER: &str = "HAS.LTD";
pub const DEFAULT_AUDIENCE: &str = "HAS.LTD";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
    pub jti: Uuid,
    pub sid: Uuid,
    pub auth_version: i64,
    pub role: String,
    pub scope: String,
}

pub struct JwtKeys {
    active_kid: String,
    active_signer: EncodingKey,
    verifiers: HashMap<String, DecodingKey>,
    issuer: String,
    audience: String,
}

impl JwtKeys {
    /// Loads RSA PEM keypairs from the specified directory.
    ///
    /// Key identification convention:
    /// - Files matching `<kid>.pem` containing PKCS#8 / PKCS#1 private keys are loaded as signers (`EncodingKey`).
    /// - Files matching `<kid>.pub.pem` containing SPKI public keys are loaded as verifiers (`DecodingKey`).
    /// - If a `<kid>.pem` private key is present without a corresponding `.pub.pem`, the SPKI public key is
    ///   derived directly from the private key so the keypair can also verify tokens.
    /// - Signer selection: The highest `kid` (evaluated numerically if integer, or lexicographically) is
    ///   selected as the active signer.
    pub fn load_from_dir(path: &Path) -> Result<Self, CryptoError> {
        if !path.exists() || !path.is_dir() {
            return Err(CryptoError::ConfigError(format!(
                "JWT key directory does not exist or is not a directory: {}",
                path.display()
            )));
        }

        let mut signers: HashMap<String, EncodingKey> = HashMap::new();
        let mut verifiers: HashMap<String, DecodingKey> = HashMap::new();

        let entries = std::fs::read_dir(path).map_err(|e| CryptoError::IoError(e))?;

        for entry in entries {
            let entry = entry.map_err(|e| CryptoError::IoError(e))?;
            let file_path = entry.path();
            if !file_path.is_file() {
                continue;
            }

            let file_name = match file_path.file_name().and_then(|s| s.to_str()) {
                Some(name) => name,
                None => continue,
            };

            let content = std::fs::read(&file_path).map_err(|e| CryptoError::IoError(e))?;

            if file_name.ends_with(".pub.pem") {
                let kid = file_name.trim_end_matches(".pub.pem").to_string();
                let decoding_key = DecodingKey::from_rsa_pem(&content)
                    .map_err(|e| CryptoError::JwtError(format!("failed to parse public key {}: {}", file_name, e)))?;
                verifiers.insert(kid, decoding_key);
            } else if file_name.ends_with(".pem") {
                let kid = file_name.trim_end_matches(".pem").to_string();
                let encoding_key = EncodingKey::from_rsa_pem(&content)
                    .map_err(|e| CryptoError::JwtError(format!("failed to parse private key {}: {}", file_name, e)))?;
                signers.insert(kid.clone(), encoding_key);

                // If public key is not already loaded for this kid, derive it from private key content
                if !verifiers.contains_key(&kid) {
                    if let Ok(decoding_key) = DecodingKey::from_rsa_pem(&content) {
                        verifiers.insert(kid, decoding_key);
                    }
                }
            }
        }

        if signers.is_empty() {
            return Err(CryptoError::ConfigError(format!(
                "no RSA private keys found in directory: {}",
                path.display()
            )));
        }

        // Deterministic active key selection: highest kid
        let mut sorted_kids: Vec<String> = signers.keys().cloned().collect();
        sorted_kids.sort_by(|a, b| {
            match (a.parse::<u64>(), b.parse::<u64>()) {
                (Ok(num_a), Ok(num_b)) => num_a.cmp(&num_b),
                _ => a.cmp(b),
            }
        });

        let active_kid = sorted_kids.last().cloned().ok_or_else(|| {
            CryptoError::ConfigError("failed to select active signing key".to_string())
        })?;

        let active_signer = signers.remove(&active_kid).ok_or_else(|| {
            CryptoError::ConfigError("missing active signer".to_string())
        })?;

        Ok(Self {
            active_kid,
            active_signer,
            verifiers,
            issuer: DEFAULT_ISSUER.to_string(),
            audience: DEFAULT_AUDIENCE.to_string(),
        })
    }

    /// Signs an access token Claims payload using RS256 and the deterministic active private key.
    pub fn sign_access_token(&self, claims: &Claims) -> Result<String, CryptoError> {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.active_kid.clone());

        jsonwebtoken::encode(&header, claims, &self.active_signer)
            .map_err(|e| CryptoError::JwtError(e.to_string()))
    }

    /// Verifies an RS256 access token.
    ///
    /// Validation rules enforced:
    /// - Rejects `alg=none`, `HS256`, or any algorithm other than RS256.
    /// - Rejects tokens missing the `kid` header.
    /// - Rejects tokens referencing an unknown `kid`.
    /// - Validates cryptographic RS256 signature against the matching public key.
    /// - Validates `exp` (expiration), `nbf` (not before), `iss` (issuer), and `aud` (audience).
    pub fn verify_access_token(&self, token: &str) -> Result<Claims, CryptoError> {
        let header = match jsonwebtoken::decode_header(token) {
            Ok(h) => h,
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("unknown variant") || err_str.contains("none") {
                    return Err(CryptoError::UnsupportedAlgorithm);
                }
                return Err(CryptoError::JwtError(err_str));
            }
        };

        // 1. Enforce RS256 only
        if header.alg != Algorithm::RS256 {
            return Err(CryptoError::UnsupportedAlgorithm);
        }

        // 2. Enforce presence of kid header
        let kid = header.kid.as_deref().ok_or(CryptoError::MissingKeyId)?;

        // 3. Lookup public key for kid
        let verifier = self.verifiers.get(kid).ok_or_else(|| {
            CryptoError::UnknownKeyId(kid.to_string())
        })?;

        // 4. Configure strict claims validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.algorithms = vec![Algorithm::RS256];
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        validation.validate_exp = true;
        validation.validate_nbf = true;

        // 5. Decode and cryptographically verify signature and claims
        let token_data = jsonwebtoken::decode::<Claims>(token, verifier, &validation)
            .map_err(CryptoError::Jwt)?;

        Ok(token_data.claims)
    }

    /// Returns the currently active kid used for signing.
    pub fn active_kid(&self) -> &str {
        &self.active_kid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    use std::io::Write;

    // Test RSA Keypair 1 (kid=1)
    const RSA_KEY_1_PRIV: &str = "-----BEGIN PRIVATE KEY-----\n\
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQCz1XdOBXy00DFK\n\
Tw30gHH6dkvGp7dXUKnxjDrhGXWcrKyPFZXj+6jJfuNJ7Vr0kkPht4UdV5NwJDAQ\n\
FCtguxeMDXZck76ShrGbteZyO3TinNgF/xlfS5/+ogqjnCDgfoevlf7MPBM+/6Kq\n\
7cCXADcMxt3lQKEf7DjqdBCnZSb2BVtk3oK+5B4Lj+SuDeYIQRp6lq3P5c9rznHA\n\
0hF0s/+i6vjVWwr7VTZyJN+qpvKMOzuEZH4uu7CzuYjTn9sBDZ9vkee0MtsXasOq\n\
maJd9doseGLvUkqikt20ygk1KlmJiGZ8jg5NE0apr9v5q/pzsqfeFskT+ViM9viW\n\
FoBE6c0ZAgMBAAECggEACkdvVFRX9nXX+2ldYNZerXRJjIdVoXYI4LelPeyYnmX4\n\
Tw1qzec6lJneIAGjK0uY+bPltfGliclnveOJPzQyelzioRQI1pNAX6gcDfd8DN/u\n\
MRmNt7a45HMarJNmBa71zoGCWOYuK3GtOJvyYC8w9lf/ovT4wLxd3HjZ4teTTIiV\n\
Ox1PltDAMGShkDeqzsXFYxePtqUEBSNfUQaDJQ18jmgXcVRAdGdJfM5U1tlqLu++\n\
GtL3qqbb3oTGTTapSTWl6gVE2G7ryAp1+nK50BWfAwcrwvvyklg/qU1jLSrGmeOj\n\
RTV4bjCCb2D7Z/yRdiYYywYobnql40jDoY/2iCQyIQKBgQDqTBxGUwTRdg83fX6y\n\
FthCp4UZf8CGEmJ5RrAL5q1ObKzwi1xRk4IlnMmRdZC/WaQ5c6HkonlUmgmUELIF\n\
dL/4CwbZHw2zH9Iq5xQ3or/uq+WXZwCHS9qAsmIv0tpy11xRB1HppG1pDRakOs+G\n\
ANxAdNpLdhL09Wz6fFhprMLbhQKBgQDEfd1AGmNmM5WXZlOta7Kjjdgee0lTCwJs\n\
WpJcPTSkpQt4rcqqDX9J9X0W6IdvuPNU/bBAUotyjun9GYSOntPuwwtW5/0YLu8/\n\
/zOqO1FRSKi/FYdfIgrqxr3sni5LPxJyEYuUE1R/uIVYe3rnWMMRXQHBgLaptoVI\n\
boLmVugNhQKBgQCkncEn7+BswsRCOpwvyDk967FqjwXXrUce6nl6gyK0/Uos392H\n\
O1qvzo8Y8j83EmZ+Ly5KE5WnyWp9a5WIm9TdE/JFzrmzZuIlXqInzOOJb6EqVSZb\n\
6QGmdq7h5wWwtnuYOspknDB7JMUJsoBoKGaXqwJNDrKXBXECNSdxZ/6oeQKBgAwt\n\
uO7mM87714PAxeLzz+ivO/v3rD3HaukIXBX3fTHPQPzD2NHdcExHOO+YjfxwvDRC\n\
XRA/jYEWNo6HbF3inkQoNqW2T0uWbSHMUYl1j764AC2HyhIta/vNQaxOyZPEXwJO\n\
8BXHdO9NooYvcRixubVzzE4QCkJOm8g3gUmBjXR5AoGBAIsihx6rQ1b0UpeRTJI3\n\
CfMFBXOnznlFGoB7TjicxQr5u3G11FIJk+Fdysu3ZS+OYJe7YQgLzEeSVZDdmEgw\n\
/HdiRPTvn5wckVnUIVlEXvP1V1YhDSvBuz8SFXf/jWMr0VgCqpfk7Ej9nZPt6uo3\n\
BMAt5UAaCdBVEfvQn2LtunKC\n\
-----END PRIVATE KEY-----";

    const RSA_KEY_1_PUB: &str = "-----BEGIN PUBLIC KEY-----\n\
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAs9V3TgV8tNAxSk8N9IBx\n\
+nZLxqe3V1Cp8Yw64Rl1nKysjxWV4/uoyX7jSe1a9JJD4beFHVeTcCQwEBQrYLsX\n\
jA12XJO+koaxm7Xmcjt04pzYBf8ZX0uf/qIKo5wg4H6Hr5X+zDwTPv+iqu3AlwA3\n\
DMbd5UChH+w46nQQp2Um9gVbZN6CvuQeC4/krg3mCEEaepatz+XPa85xwNIRdLP/\n\
our41VsK+1U2ciTfqqbyjDs7hGR+Lruws7mI05/bAQ2fb5HntDLbF2rDqpmiXfXa\n\
LHhi71JKopLdtMoJNSpZiYhmfI4OTRNGqa/b+av6c7Kn3hbJE/lYjPb4lhaAROnN\n\
GQIDAQAB\n\
-----END PUBLIC KEY-----";

    // Test RSA Keypair 2 (kid=2, active signer)
    const RSA_KEY_2_PRIV: &str = "-----BEGIN PRIVATE KEY-----\n\
MIIEvwIBADANBgkqhkiG9w0BAQEFAASCBKkwggSlAgEAAoIBAQDIU9ySwJCll0aA\n\
+1RVHZAeEEALYveuHuzz9DZg/N8QJh+rJjiLJye3BXap/J33JNtDAUlf3Ty0OxEY\n\
E+nVGF9fGzPDBVCG7a7CHAFcdivD7kJVAWI9IvmEz8WoZUTjJ9Es3lAMWylTu6Hv\n\
NIrDw1hovjwGFcn+CQfAG/e5YcgkxYveDeaTfPASG3lP56BuzWdOio241WaZqR69\n\
9MMea/LHCLOuB5IkRk8RGL+x5Acua9DfsM6tMmP/MZ0093cVPn04rA2vsvfdwMxr\n\
B/KA97KKy1CCEQP8jzmpZ8JzFfmX8SiL92ScX/IaFwo4lLqwYIr7H/DpfBe8gsQV\n\
4eJNBo4ZAgMBAAECggEABOXBoLABlp4d3e6htiHT7tuObfDVwGLMFR6RRe+nW5TW\n\
Wp1Djd8Ez6rGpvB7Tf+EzEIvwJQZ2of5+qJBqdaPvf/zX6HQQTeLmZOCHYLGk7f4\n\
v20E8VzFkp+FPqu4zSBhXY+oT5WwIJZ3sU+cp7Ki2qRcTDj0EEhj0VNQYWP7IYSh\n\
8T57xQkUnomHBEkvpXfF3BNkXvZwTslA50Gj+HQxEu9AqGzgHw5jDkj5qSpVqLmU\n\
kSMoOCe/mNsJbvOKvuP1BkxIEwOvPEHEm3BRcABwB/ta26Lwj09+22A5h4pVCEwl\n\
vAsCE0hfP2UkVuEFuWGzRyfrnAofpC6RvT7RfpbvAQKBgQDnAV5dIOGDZ0hNHtea\n\
iRstti+VWtl7u9Ql0wVRH+3LWoKu+QV6XIHfnHka1Jj+hwskWt1bmDRGYcx75h6T\n\
feRK7fwFk/ipTkSiqE0jDKxPg/ula+Bd7Zuv9Bxymtug636QX1z765yY0oCvY/ZH\n\
aKNkJfxuEOl2obRXue2KhTGzAQKBgQDeAL9uycHpy7WZH9+1DQWBGcUQy/FXlJwe\n\
2AH9dASY6esOtiJ19GJpqabxTZeYodqGk450USXp6NfCTQ9vgJP2sXTwugBRI78b\n\
FnxvrpSnXXuu5WjkmUWEmjtXqHnY2yC1gNdmL/nV2gQnBMm1GtAetqR1E/ye6YVK\n\
SFUT4eMTGQKBgQC4krNfHcKSR5fRZu9GHn6Ey6jBYVGcN7uObUk2RJivfqYkP/xM\n\
hFWQ4WkyMuP7mtMgPSnY4YgDJGlKJdgQaOpbpENWAaERReQ5Qtg/RQT4k7AhwEwJ\n\
QKzArf984pbRwNfjaw7UirEYujjv6qMA8255upAzMXgYvaNm5LwTjdDkAQKBgQDE\n\
kC7vfi52voqk8MbXfJcSDuogjCK9Y/Szjph8fDwR1JPua6+pdt+oJLGO2B/Q4gYl\n\
wAKS1MLYD/Txsd9n35WjCM680kOzlA4Cgu9cx1wRsEDxw2gTd+5IkXLXk3GHY8Xb\n\
Tx/o97Oi+K1Sive5per+NsL4cuDrJ2KAOfkyaH/xCQKBgQDFUCObuG9l8qLSLOqv\n\
aj+VF8Mp70sciR08V/M0OCIXPnHZ64oK88PJi4Z4Qtg7wBiCBD18rKIJVbSF7O+W\n\
E9XhFZuQx7jh34c/85LDig9c6gXO3HY64Wei0DFXmMYKpTq8oHf3CiDCI/5bvPw0\n\
JOTRbNiOBKnxRnyggNzve4zGRA==\n\
-----END PRIVATE KEY-----";

    const RSA_KEY_2_PUB: &str = "-----BEGIN PUBLIC KEY-----\n\
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAyFPcksCQpZdGgPtUVR2Q\n\
HhBAC2L3rh7s8/Q2YPzfECYfqyY4iycntwV2qfyd9yTbQwFJX908tDsRGBPp1Rhf\n\
XxszwwVQhu2uwhwBXHYrw+5CVQFiPSL5hM/FqGVE4yfRLN5QDFspU7uh7zSKw8NY\n\
aL48BhXJ/gkHwBv3uWHIJMWL3g3mk3zwEht5T+egbs1nToqNuNVmmakevfTDHmvy\n\
xwizrgeSJEZPERi/seQHLmvQ37DOrTJj/zGdNPd3FT59OKwNr7L33cDMawfygPey\n\
istQghED/I85qWfCcxX5l/Eoi/dknF/yGhcKOJS6sGCK+x/w6XwXvILEFeHiTQaO\n\
GQIDAQAB\n\
-----END PUBLIC KEY-----";

    fn setup_test_jwt_keys() -> (tempfile::TempDir, JwtKeys) {
        let dir = tempfile::tempdir().expect("tempdir creation should succeed");

        let p1 = dir.path().join("1.pem");
        let mut f1 = std::fs::File::create(&p1).unwrap();
        f1.write_all(RSA_KEY_1_PRIV.as_bytes()).unwrap();

        let pub1 = dir.path().join("1.pub.pem");
        let mut fpub1 = std::fs::File::create(&pub1).unwrap();
        fpub1.write_all(RSA_KEY_1_PUB.as_bytes()).unwrap();

        let p2 = dir.path().join("2.pem");
        let mut f2 = std::fs::File::create(&p2).unwrap();
        f2.write_all(RSA_KEY_2_PRIV.as_bytes()).unwrap();

        let pub2 = dir.path().join("2.pub.pem");
        let mut fpub2 = std::fs::File::create(&pub2).unwrap();
        fpub2.write_all(RSA_KEY_2_PUB.as_bytes()).unwrap();

        let jwt_keys = JwtKeys::load_from_dir(dir.path()).expect("keys must load successfully");
        (dir, jwt_keys)
    }

    fn sample_claims(now: i64) -> Claims {
        Claims {
            sub: "user-123".to_string(),
            iss: DEFAULT_ISSUER.to_string(),
            aud: DEFAULT_AUDIENCE.to_string(),
            exp: now + 900, // 15 min lifetime
            iat: now,
            nbf: now - 5,
            jti: Uuid::new_v4(),
            sid: Uuid::new_v4(),
            auth_version: 1,
            role: "CUSTOMER".to_string(),
            scope: "access".to_string(),
        }
    }

    #[test]
    fn test_jwt_happy_path() {
        let (_dir, keys) = setup_test_jwt_keys();
        assert_eq!(keys.active_kid(), "2", "Highest kid (2) must be selected as active signer");

        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let claims = sample_claims(now);

        let token = keys.sign_access_token(&claims).expect("signing must succeed");
        let decoded = keys.verify_access_token(&token).expect("verification must succeed");

        assert_eq!(decoded.sub, claims.sub);
        assert_eq!(decoded.role, claims.role);
        assert_eq!(decoded.auth_version, claims.auth_version);
    }

    #[test]
    fn test_jwt_expired_token_rejected() {
        let (_dir, keys) = setup_test_jwt_keys();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let mut claims = sample_claims(now);
        claims.exp = now - 3600; // Expired 1 hour ago
        claims.nbf = now - 7200;

        let token = keys.sign_access_token(&claims).expect("signing must succeed");
        let res = keys.verify_access_token(&token);

        match res {
            Err(CryptoError::Jwt(e)) => assert_eq!(*e.kind(), jsonwebtoken::errors::ErrorKind::ExpiredSignature),
            other => panic!("Expected expired token error, got {:?}", other),
        }
    }

    #[test]
    fn test_jwt_wrong_issuer_rejected() {
        let (_dir, keys) = setup_test_jwt_keys();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let mut claims = sample_claims(now);
        claims.iss = "MALICIOUS_ISSUER".to_string();

        let token = keys.sign_access_token(&claims).expect("signing must succeed");
        let res = keys.verify_access_token(&token);

        match res {
            Err(CryptoError::Jwt(e)) => assert_eq!(*e.kind(), jsonwebtoken::errors::ErrorKind::InvalidIssuer),
            other => panic!("Expected wrong issuer error, got {:?}", other),
        }
    }

    #[test]
    fn test_jwt_wrong_audience_rejected() {
        let (_dir, keys) = setup_test_jwt_keys();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let mut claims = sample_claims(now);
        claims.aud = "WRONG_AUDIENCE".to_string();

        let token = keys.sign_access_token(&claims).expect("signing must succeed");
        let res = keys.verify_access_token(&token);

        match res {
            Err(CryptoError::Jwt(e)) => assert_eq!(*e.kind(), jsonwebtoken::errors::ErrorKind::InvalidAudience),
            other => panic!("Expected wrong audience error, got {:?}", other),
        }
    }

    #[test]
    fn test_jwt_tampered_payload_rejected() {
        let (_dir, keys) = setup_test_jwt_keys();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let claims = sample_claims(now);

        let token = keys.sign_access_token(&claims).expect("signing must succeed");
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);

        // Tamper with payload segment
        let mut tampered_payload = parts[1].to_string();
        if tampered_payload.ends_with('A') {
            tampered_payload.pop();
            tampered_payload.push('B');
        } else {
            tampered_payload.pop();
            tampered_payload.push('A');
        }

        let tampered_token = format!("{}.{}.{}", parts[0], tampered_payload, parts[2]);
        let res = keys.verify_access_token(&tampered_token);
        assert!(res.is_err(), "Tampered payload must be rejected");
    }

    #[test]
    fn test_jwt_alg_none_rejected() {
        let (_dir, keys) = setup_test_jwt_keys();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let claims = sample_claims(now);

        // Hand-craft raw JWT with alg=none
        let header_json = r#"{"alg":"none","typ":"JWT","kid":"2"}"#;
        let claims_json = serde_json::to_string(&claims).unwrap();

        let header_b64 = URL_SAFE_NO_PAD.encode(header_json.as_bytes());
        let claims_b64 = URL_SAFE_NO_PAD.encode(claims_json.as_bytes());

        let forged_token = format!("{}.{}.", header_b64, claims_b64);
        let res = keys.verify_access_token(&forged_token);

        match res {
            Err(CryptoError::UnsupportedAlgorithm) => {}
            other => panic!("Expected UnsupportedAlgorithm error for alg=none, got {:?}", other),
        }
    }

    #[test]
    fn test_jwt_hs256_with_rs256_public_key_rejected() {
        let (_dir, keys) = setup_test_jwt_keys();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let claims = sample_claims(now);

        // Hand-craft token signed with HS256 using public key bytes as HMAC key
        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some("2".to_string());

        let hmac_key = jsonwebtoken::EncodingKey::from_secret(RSA_KEY_2_PUB.as_bytes());
        let hs256_token = jsonwebtoken::encode(&header, &claims, &hmac_key).unwrap();

        let res = keys.verify_access_token(&hs256_token);
        match res {
            Err(CryptoError::UnsupportedAlgorithm) => {}
            other => panic!("Expected UnsupportedAlgorithm for HS256 token, got {:?}", other),
        }
    }

    #[test]
    fn test_jwt_unknown_kid_rejected() {
        let (_dir, keys) = setup_test_jwt_keys();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let claims = sample_claims(now);

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("non_existent_kid_999".to_string());

        let signer = jsonwebtoken::EncodingKey::from_rsa_pem(RSA_KEY_2_PRIV.as_bytes()).unwrap();
        let token = jsonwebtoken::encode(&header, &claims, &signer).unwrap();

        let res = keys.verify_access_token(&token);
        match res {
            Err(CryptoError::UnknownKeyId(kid)) => assert_eq!(kid, "non_existent_kid_999"),
            other => panic!("Expected UnknownKeyId error, got {:?}", other),
        }
    }

    #[test]
    fn test_jwt_missing_kid_rejected() {
        let (_dir, keys) = setup_test_jwt_keys();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let claims = sample_claims(now);

        let mut header = Header::new(Algorithm::RS256);
        header.kid = None; // Deliberately omit kid

        let signer = jsonwebtoken::EncodingKey::from_rsa_pem(RSA_KEY_2_PRIV.as_bytes()).unwrap();
        let token = jsonwebtoken::encode(&header, &claims, &signer).unwrap();

        let res = keys.verify_access_token(&token);
        match res {
            Err(CryptoError::MissingKeyId) => {}
            other => panic!("Expected MissingKeyId error, got {:?}", other),
        }
    }

    #[test]
    fn test_jwt_scope_access_vs_mfa_pending() {
        let (_dir, keys) = setup_test_jwt_keys();
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;

        // 1. Token with scope == "access"
        let mut claims_access = sample_claims(now);
        claims_access.scope = "access".to_string();
        let token_access = keys.sign_access_token(&claims_access).expect("signing must succeed");
        let decoded_access = keys.verify_access_token(&token_access).expect("verification must succeed");
        assert_eq!(decoded_access.scope, "access");

        // 2. Token with scope == "mfa_pending"
        let mut claims_mfa = sample_claims(now);
        claims_mfa.scope = "mfa_pending".to_string();
        let token_mfa = keys.sign_access_token(&claims_mfa).expect("signing must succeed");
        let decoded_mfa = keys.verify_access_token(&token_mfa).expect("verification must succeed");
        assert_eq!(decoded_mfa.scope, "mfa_pending");
    }
}
