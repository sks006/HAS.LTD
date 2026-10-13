-- scripts/promote_admin.sql
-- Operational administrative promotion script.
--
-- SECURITY MANDATE:
-- This script must be executed manually and explicitly by an infrastructure operator.
-- Never run automated role elevations inside schema migrations or unattended scripts.
-- Every role elevation updates the user's role to 'ADMIN' and automatically bumps auth_version
-- via trigger (trg_users_bump_auth_version), invalidating any existing active sessions.
--
-- Usage:
--   psql -d has -v target_email="'admin@has.ltd'" -f scripts/promote_admin.sql
-- Or inside a psql session:
--   \set target_email 'admin@has.ltd'
--   \i scripts/promote_admin.sql

UPDATE users
SET role = 'ADMIN',
    updated_at = NOW()
WHERE email = :target_email
   OR email_normalized = LOWER(TRIM(:target_email));
