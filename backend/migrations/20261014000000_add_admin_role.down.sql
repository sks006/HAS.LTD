-- Migration 20261014000000_add_admin_role.down.sql
-- Revert role CHECK constraint

ALTER TABLE users DROP CONSTRAINT IF EXISTS chk_users_role;
