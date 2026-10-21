-- Migration 20261014000100_auth_schema.down.sql
-- Rollback for auth_schema
--
-- WARNING: DESTRUCTIVE ROLLBACK.
-- This rollback drops credentials, sessions, refresh tokens, MFA credentials, and audit logs.
-- Password hashes for local accounts are best-effort restored to users.password_hash before credentials is dropped.
-- Social logins, webauthn credentials, and MFA secrets will be permanently lost.

-- 1. Restore password_hash on users and repopulate from credentials before dropping table
ALTER TABLE users ADD COLUMN IF NOT EXISTS password_hash VARCHAR(255);

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'credentials') THEN
        UPDATE users u
        SET password_hash = c.secret_hash
        FROM credentials c
        WHERE c.user_id = u.id AND c.kind = 'password' AND c.secret_hash IS NOT NULL;
    END IF;
END $$;

-- 2. Drop auth infrastructure tables
DROP TABLE IF EXISTS audit_log;
DROP TABLE IF EXISTS mfa_credentials;
DROP TABLE IF EXISTS refresh_tokens;
DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS credentials;

-- 3. Revert alterations on users table
ALTER TABLE users 
    DROP CONSTRAINT IF EXISTS chk_users_status,
    DROP CONSTRAINT IF EXISTS uq_users_email_normalized,
    DROP COLUMN IF EXISTS email_normalized,
    DROP COLUMN IF EXISTS status,
    DROP COLUMN IF EXISTS auth_version,
    DROP COLUMN IF EXISTS metadata;

DROP TYPE IF EXISTS credential_kind_enum;
DROP TYPE IF EXISTS user_status_enum;
