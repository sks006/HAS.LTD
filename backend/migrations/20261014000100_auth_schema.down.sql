-- Migration 20261014000100_auth_schema.down.sql
-- Rollback for auth_schema

DROP TABLE IF EXISTS audit_log;
DROP TABLE IF EXISTS mfa_credentials;
DROP TABLE IF EXISTS refresh_tokens;
DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS credentials;

-- Restore password_hash on users if rolling back
ALTER TABLE users ADD COLUMN IF NOT EXISTS password_hash VARCHAR(255);

ALTER TABLE users 
    DROP CONSTRAINT IF EXISTS uq_users_email_normalized,
    DROP COLUMN IF EXISTS email_normalized,
    DROP COLUMN IF EXISTS status,
    DROP COLUMN IF EXISTS auth_version,
    DROP COLUMN IF EXISTS metadata;

DROP TYPE IF EXISTS credential_kind_enum;
DROP TYPE IF EXISTS user_status_enum;
