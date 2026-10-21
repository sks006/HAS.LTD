# Runbook: Mass Logout Incident Investigation & Recovery

## 1. Alert Name and Short Description
- **Alert Name**: `AuthMassLogoutSpike`
- **Short Description**: Unusually high spike in global logout operations (`auth.logout_all`) or widespread session invalidation via `users.auth_version` increments, causing mass user session disconnections.

## 2. Symptoms
- **User Impact**: Thousands of active users receive HTTP 401 `token_revoked` or `account_suspended` simultaneously across mobile apps and web frontends, forcing full re-authentication.
- **Dashboard Indications**:
  - `Auth Security`: "Security Events: Admin / Global Actions" panel shows vertical spike for `auth.logout_all`.
  - `Auth Health`: Steep surge in `/auth/login` attempts as logged-out users attempt to log back in.
  - Surge in `auth_middleware_rejection_total{reason="token_revoked"}`.

## 3. Diagnosis Steps
1. **Query Audit Log for Mass Logout Triggers**:
   ```sql
   -- Find callers initiating auth.logout_all within the last 30 minutes
   SELECT user_id, ip_address, user_agent, metadata, created_at
   FROM audit_log
   WHERE event = 'auth.logout_all'
     AND created_at > NOW() - INTERVAL '30 minutes'
   ORDER BY created_at DESC
   LIMIT 100;
   ```
2. **Inspect auth_version Increments Across the User Table**:
   ```sql
   -- Identify users with recently bumped auth_version
   SELECT id, email, role, status, auth_version, updated_at
   FROM users
   WHERE updated_at > NOW() - INTERVAL '30 minutes'
   ORDER BY updated_at DESC
   LIMIT 50;
   ```
3. **Verify Redis Cache Invalidation State**:
   ```bash
   # Connect to Redis cluster and check if keys are flushed or mass-invalidated
   redis-cli -h $REDIS_HOST -p $REDIS_PORT INFO stats
   # Sample random auth_version keys
   redis-cli -h $REDIS_HOST -p $REDIS_PORT KEYS "auth_version:*" | head -n 20
   ```
4. **Determine Administrative vs Client-Driven Cause**:
   - Check if an admin API script ran an unbounded `UPDATE users SET auth_version = auth_version + 1`.
   - Check if a credential stuffing or password reset campaign triggered systematic session revocations.

## 4. Mitigation (Immediate Action)
1. **Block Malicious Emitter IP or Client**:
   If a single compromised admin token or botnet IP is issuing mass `/auth/logout-all`:
   ```bash
   # Block IP at reverse proxy / Cloudflare / iptables
   sudo iptables -I INPUT -s $MALICIOUS_IP -j DROP
   ```
2. **Throttle Inbound Re-authentication Surge**:
   Prevent database connection pool saturation or Argon2 gate exhaustion:
   ```bash
   # Verify Argon2 queue depth on Auth Health dashboard
   # If queue depth > 32, scale auth API worker pods:
   kubectl scale deployment has-auth-api --replicas=12
   ```
3. **Warm Redis Auth Version Cache**:
   Prevent cache-miss stampedes to PostgreSQL `SELECT auth_version FROM users`:
   ```bash
   # Check DB CPU and connection metrics; if elevated, ensure hot users are populated
   ```

## 5. Root Cause Categories
- **Admin Operator Error**: Bulk script executed an unconstrained update on `users` table or invoked administrative logout endpoints indiscriminately.
- **Security Automated Response**: Automated threat mitigation rule triggered global logout on broad IP range.
- **Client Bug**: A mobile application loop incorrectly called `/auth/logout-all` on token refresh retry failure.

## 6. Rollback Procedure
Session revocations are cryptographically non-reversible (once refresh tokens have `revoked_at` set and `auth_version` is bumped, old access tokens are permanently invalid per design):
1. Users must undergo standard login (`/auth/login`) or MFA challenge.
2. Ensure email OTP and Resend dispatch services are scaled to handle OTP surges:
   ```bash
   # Inspect email dispatch queue health:
   curl -s http://localhost:8080/metrics | grep auth_email_dispatch
   ```
3. If an erroneous migration script is actively running, kill the database process:
   ```sql
   SELECT pg_terminate_backend(pid)
   FROM pg_stat_activity
   WHERE query LIKE '%auth_version%' AND pid <> pg_backend_pid();
   ```

## 7. Related Runbooks
- `docs/runbooks/postgres-failover.md`
- `docs/runbooks/suspected-account-breach.md`
- `docs/runbooks/redis-outage.md`
