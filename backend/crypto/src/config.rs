// backend/crypto/src/config.rs

use std::path::PathBuf;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

use crate::errors::CryptoError;
use crate::password::{DEFAULT_M_COST, DEFAULT_OUTPUT_LEN, DEFAULT_P_COST, DEFAULT_T_COST};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Argon2Params {
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
    pub output_len: usize,
}

impl Default for Argon2Params {
    fn default() -> Self {
        Self {
            m_cost: DEFAULT_M_COST,
            t_cost: DEFAULT_T_COST,
            p_cost: DEFAULT_P_COST,
            output_len: DEFAULT_OUTPUT_LEN,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CryptoConfig {
    pub jwt_dir: PathBuf,
    pub totp_key_path: PathBuf,
    pub argon2_params: Argon2Params,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            jwt_dir: PathBuf::from(".secrets/jwt"),
            totp_key_path: PathBuf::from(".secrets/totp.key"),
            argon2_params: Argon2Params::default(),
        }
    }
}

impl CryptoConfig {
    /// Constructs a Figment provider chain establishing configuration precedence:
    /// Environment Variables (`APP_CRYPTO__*`) > Configuration File (`config.toml`) > Compiled Defaults.
    pub fn figment() -> Figment {
        Figment::new()
            .merge(Serialized::defaults(CryptoConfig::default()))
            .merge(Toml::file("config.toml").nested())
            .merge(Env::prefixed("APP_CRYPTO__").split("__"))
    }

    /// Loads and strictly validates configuration. Fails loudly at startup if
    /// `jwt_dir` is missing/empty or `totp_key_path` is missing/not 32 bytes.
    pub fn load() -> Result<Self, CryptoError> {
        let config: CryptoConfig = Self::figment()
            .extract()
            .map_err(|e| CryptoError::ConfigError(e.to_string()))?;

        config.validate()?;
        Ok(config)
    }

    /// Validates directory existence and key sizing.
    pub fn validate(&self) -> Result<(), CryptoError> {
        // 1. jwt_dir must exist, be a directory, and contain at least one .pem file
        if !self.jwt_dir.exists() {
            return Err(CryptoError::ConfigError(format!(
                "jwt_dir does not exist: {}",
                self.jwt_dir.display()
            )));
        }
        if !self.jwt_dir.is_dir() {
            return Err(CryptoError::ConfigError(format!(
                "jwt_dir is not a directory: {}",
                self.jwt_dir.display()
            )));
        }

        let entries = std::fs::read_dir(&self.jwt_dir)
            .map_err(|e| CryptoError::IoError(e))?;
        let mut has_pem = false;
        for entry in entries {
            let entry = entry.map_err(|e| CryptoError::IoError(e))?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("pem") {
                has_pem = true;
                break;
            }
        }
        if !has_pem {
            return Err(CryptoError::ConfigError(format!(
                "jwt_dir is empty or contains no .pem files: {}",
                self.jwt_dir.display()
            )));
        }

        // 2. totp_key_path must exist and be exactly 32 bytes
        if !self.totp_key_path.exists() {
            return Err(CryptoError::ConfigError(format!(
                "totp_key_path does not exist: {}",
                self.totp_key_path.display()
            )));
        }
        let metadata = std::fs::metadata(&self.totp_key_path)
            .map_err(|e| CryptoError::IoError(e))?;
        if metadata.len() != 32 {
            return Err(CryptoError::ConfigError(format!(
                "totp_key_path must be exactly 32 bytes, found {} bytes: {}",
                metadata.len(),
                self.totp_key_path.display()
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_gitignore_contains_secrets_dir_before_init_secrets_runs() {
        // Find .gitignore in repo root (search upwards from current dir or cargo manifest dir)
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root_gitignore = manifest_dir.join("../../.gitignore");

        let content = std::fs::read_to_string(&root_gitignore)
            .expect("workspace .gitignore must exist");

        let has_secrets = content.lines().any(|line| line.trim() == ".secrets/" || line.trim() == ".secrets");
        assert!(
            has_secrets,
            ".gitignore must contain .secrets/ BEFORE init_secrets.sh generates key material"
        );
    }

    #[test]
    fn test_loader_fails_if_jwt_dir_missing() {
        let dir = tempfile::tempdir().unwrap();
        let key_path = dir.path().join("totp.key");
        std::fs::write(&key_path, [0u8; 32]).unwrap();

        let config = CryptoConfig {
            jwt_dir: dir.path().join("non_existent_jwt_dir"),
            totp_key_path: key_path,
            argon2_params: Argon2Params::default(),
        };

        let res = config.validate();
        match res {
            Err(CryptoError::ConfigError(msg)) => assert!(msg.contains("jwt_dir does not exist")),
            other => panic!("Expected ConfigError for missing jwt_dir, got {:?}", other),
        }
    }

    #[test]
    fn test_loader_fails_if_jwt_dir_empty() {
        let dir = tempfile::tempdir().unwrap();
        let empty_jwt_dir = dir.path().join("empty_jwt");
        std::fs::create_dir(&empty_jwt_dir).unwrap();

        let key_path = dir.path().join("totp.key");
        std::fs::write(&key_path, [0u8; 32]).unwrap();

        let config = CryptoConfig {
            jwt_dir: empty_jwt_dir,
            totp_key_path: key_path,
            argon2_params: Argon2Params::default(),
        };

        let res = config.validate();
        match res {
            Err(CryptoError::ConfigError(msg)) => assert!(msg.contains("empty or contains no .pem files")),
            other => panic!("Expected ConfigError for empty jwt_dir, got {:?}", other),
        }
    }

    #[test]
    fn test_loader_fails_if_totp_key_not_32_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let jwt_dir = dir.path().join("jwt");
        std::fs::create_dir(&jwt_dir).unwrap();
        let mut pem = std::fs::File::create(jwt_dir.join("1.pem")).unwrap();
        pem.write_all(b"fake pem").unwrap();

        // 16 bytes instead of 32 bytes
        let short_key_path = dir.path().join("totp.key");
        std::fs::write(&short_key_path, [0u8; 16]).unwrap();

        let config = CryptoConfig {
            jwt_dir,
            totp_key_path: short_key_path,
            argon2_params: Argon2Params::default(),
        };

        let res = config.validate();
        match res {
            Err(CryptoError::ConfigError(msg)) => assert!(msg.contains("must be exactly 32 bytes")),
            other => panic!("Expected ConfigError for invalid totp.key size, got {:?}", other),
        }
    }

    #[test]
    fn test_loader_succeeds_when_valid() {
        let dir = tempfile::tempdir().unwrap();
        let jwt_dir = dir.path().join("jwt");
        std::fs::create_dir(&jwt_dir).unwrap();
        let mut pem = std::fs::File::create(jwt_dir.join("1.pem")).unwrap();
        pem.write_all(b"valid pem").unwrap();

        let key_path = dir.path().join("totp.key");
        std::fs::write(&key_path, [0u8; 32]).unwrap();

        let config = CryptoConfig {
            jwt_dir,
            totp_key_path: key_path,
            argon2_params: Argon2Params::default(),
        };

        assert!(config.validate().is_ok());
    }
}
