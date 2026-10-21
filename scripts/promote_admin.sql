-- scripts/promote_admin.sql
-- Operational administrative promotion script.
--
-- SECURITY MANDATE:
-- This script must be executed manually and explicitly by an infrastructure operator.
-- Never run automated role elevations inside schema migrations or unattended scripts.
--
-- Token & Session Lifecycle on Administrative Promotion:
-- 1. Elevating the role to 'ADMIN' fires trigger `trg_users_bump_auth_version`,
--    incrementing `users.auth_version`. This immediately invalidates in-flight stateless
--    JWT access tokens (guards verify JWT claims.auth_version == users.auth_version).
-- 2. Sessions in the `sessions` table stay alive until refreshed or explicitly revoked;
--    on refresh, tokens are checked against the user's updated auth_version.
-- 3. For defense-in-depth on privileged promotion, this script explicitly revokes all active
--    sessions and refresh tokens for the user, forcing full re-authentication.
--
-- Usage:
--   psql -d has -v target_email=admin@has.ltd -f scripts/promote_admin.sql
-- Or inside a psql session:
--   \set target_email 'admin@has.ltd'
--   \i scripts/promote_admin.sql

-- 1. Promote user and trigger auth_version bump
UPDATE users
SET role = 'ADMIN',
    updated_at = NOW()
WHERE email_normalized = LOWER(TRIM(:'target_email'));

-- 2. Revoke all active refresh tokens for the promoted user
UPDATE refresh_tokens
SET revoked_at = NOW()
WHERE session_id IN (
    SELECT id FROM sessions 
    WHERE user_id = (SELECT id FROM users WHERE email_normalized = LOWER(TRIM(:'target_email')))
) AND revoked_at IS NULL;

-- 3. Revoke all active sessions for the promoted user
UPDATE sessions
SET revoked_at = NOW()
WHERE user_id = (SELECT id FROM users WHERE email_normalized = LOWER(TRIM(:'target_email')))
  AND revoked_at IS NULL;
