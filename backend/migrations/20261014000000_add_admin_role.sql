-- Migration 20261014000000_add_admin_role.sql
-- Forward-only migration adding ADMIN to role CHECK constraint.
-- No accounts are elevated in this migration.

-- 1. Standardize existing role casing to uppercase canonical representation
UPDATE users SET role = 'CUSTOMER' WHERE LOWER(role) = 'customer';
UPDATE users SET role = 'MODERATOR' WHERE LOWER(role) = 'moderator';
UPDATE users SET role = 'ADMIN' WHERE LOWER(role) = 'admin' OR LOWER(role) = 'superadmin';

-- 2. Add CHECK constraint guaranteeing only valid roles can be stored
ALTER TABLE users
    DROP CONSTRAINT IF EXISTS chk_users_role;

ALTER TABLE users
    ADD CONSTRAINT chk_users_role CHECK (role IN ('CUSTOMER', 'MODERATOR', 'ADMIN'));
