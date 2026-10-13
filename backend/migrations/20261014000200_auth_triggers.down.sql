-- Migration 20261014000200_auth_triggers.down.sql
-- Drop trigger and trigger function

DROP TRIGGER IF EXISTS trg_users_bump_auth_version ON users;
DROP FUNCTION IF EXISTS fn_bump_auth_version_on_status_or_role();
