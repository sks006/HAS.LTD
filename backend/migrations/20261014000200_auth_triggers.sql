-- Migration 20261014000200_auth_triggers.sql
-- Trigger to automatically increment auth_version and updated_at on status or role changes.

CREATE OR REPLACE FUNCTION fn_bump_auth_version_on_status_or_role()
RETURNS TRIGGER AS $$
BEGIN
    IF (OLD.status IS DISTINCT FROM NEW.status) OR (OLD.role IS DISTINCT FROM NEW.role) THEN
        NEW.auth_version := OLD.auth_version + 1;
        NEW.updated_at := now();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_users_bump_auth_version ON users;

CREATE TRIGGER trg_users_bump_auth_version
BEFORE UPDATE OF status, role ON users
FOR EACH ROW
EXECUTE FUNCTION fn_bump_auth_version_on_status_or_role();
