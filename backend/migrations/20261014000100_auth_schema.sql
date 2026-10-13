-- Migration 20261014000100_auth_schema.sql
-- Forward-only migration implementing core identity, credentials, sessions, refresh tokens, MFA, and audit logging.

-- 1. Create credential kind enum and user status enum
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_status_enum') THEN
        CREATE TYPE user_status_enum AS ENUM ('ACTIVE', 'SUSPENDED', 'DELETED', 'PENDING');
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'credential_kind_enum') THEN
        CREATE TYPE credential_kind_enum AS ENUM ('password', 'social', 'webauthn');
    END IF;
END $$;

-- 2. Alter users table additively to match authoritative specification
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS email_normalized VARCHAR(255),
    ADD COLUMN IF NOT EXISTS status user_status_enum NOT NULL DEFAULT 'ACTIVE',
    ADD COLUMN IF NOT EXISTS auth_version BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS metadata JSONB NOT NULL DEFAULT '{}'::jsonb;

-- Backfill email_normalized from existing email
UPDATE users
SET email_normalized = LOWER(TRIM(email))
WHERE email_normalized IS NULL;

ALTER TABLE users
    ALTER COLUMN email_normalized SET NOT NULL;

-- Enforce unique constraint on normalized email
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'uq_users_email_normalized') THEN
        ALTER TABLE users ADD CONSTRAINT uq_users_email_normalized UNIQUE (email_normalized);
    END IF;
END $$;

-- 3. Create credentials table (isolating passwords & auth factors)
CREATE TABLE IF NOT EXISTS credentials (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind credential_kind_enum NOT NULL,
    provider VARCHAR(50),
    provider_subject VARCHAR(255),
    secret_hash TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT uq_credentials_user_kind_provider UNIQUE (user_id, kind, provider)
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_credentials_provider_subject
    ON credentials (provider, provider_subject)
    WHERE provider IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_credentials_user_id
    ON credentials (user_id);

-- Backfill legacy password hashes into credentials table if password_hash exists on users
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'password_hash'
    ) THEN
        INSERT INTO credentials (id, user_id, kind, provider, provider_subject, secret_hash, created_at, updated_at)
        SELECT 
            gen_random_uuid(),
            id,
            'password'::credential_kind_enum,
            'local',
            NULL,
            password_hash,
            created_at,
            updated_at
        FROM users
        WHERE password_hash IS NOT NULL
        ON CONFLICT (user_id, kind, provider) DO NOTHING;

        -- Drop password_hash from users to eliminate credentials on users anti-pattern
        ALTER TABLE users DROP COLUMN password_hash;
    END IF;
END $$;

-- 4. Create sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_fingerprint TEXT,
    user_agent TEXT,
    ip_address INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_active
    ON sessions (user_id, revoked_at)
    WHERE revoked_at IS NULL;

-- 5. Create refresh_tokens table (with RTR and family replay detection)
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    family_id UUID NOT NULL,
    token_hash VARCHAR(64) UNIQUE NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    replaced_by_id UUID REFERENCES refresh_tokens(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_refresh_tokens_session_active
    ON refresh_tokens (session_id)
    WHERE revoked_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_refresh_tokens_family_id
    ON refresh_tokens (family_id);

-- 6. Create mfa_credentials table
CREATE TABLE IF NOT EXISTS mfa_credentials (
    id UUID PRIMARY KEY,
    user_id UUID UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind VARCHAR(20) NOT NULL DEFAULT 'totp',
    secret_encrypted BYTEA NOT NULL,
    recovery_codes_hash TEXT[] NOT NULL DEFAULT '{}',
    confirmed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 7. Create audit_log table
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    event TEXT NOT NULL,
    ip_address INET,
    user_agent TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_audit_log_user_created
    ON audit_log (user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_audit_log_event_created
    ON audit_log (event, created_at DESC);
