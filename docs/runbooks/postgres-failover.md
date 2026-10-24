# Runbook: PostgreSQL Database Failover & Connection Pool Saturation

## 1. Alert Name and Short Description
- **Alert Name**: `AuthMiddlewareInternalErrorRate`
- **Short Description**: Elevated rate of internal server errors (`auth_middleware_rejection_total{reason="internal_error"}`) exceeding 1% of total auth traffic, caused by primary database unresponsiveness, pool exhaustion, or failover transitions.

## 2. Symptoms
- **User Impact**: Inbound requests across `/auth/login`, `/auth/refresh`, and protected API endpoints return HTTP 500 Internal Server Error (`{ error: { code: "internal_error", message: "..." } }`).
- **Dashboard Indications**:
  - `Auth Health`: "Request Rate" drops or returns 500s; "DB Query Rate" collapses to 0 or flatlines.
  - Spike in `auth_middleware_rejection_total{reason="internal_error"}`.
  - Alertmanager triggers `AuthHigh5xxRate` (P0 Page).

## 3. Diagnosis Steps
1. **Check Primary PostgreSQL Database Status**:
   ```bash
   # Verify connection directly from auth node
   pg_isready -h $DB_HOST -p 5432 -U postgres -d has_auth
   ```
2. **Inspect Connection Pool Metrics & Active Backends**:
   ```sql
   -- Run on standby or primary via psql
   SELECT pid, usename, client_addr, state, wait_event_type, wait_event, query
   FROM pg_stat_activity
   WHERE datname = 'has_auth'
   ORDER BY state_change DESC
   LIMIT 30;
   ```
3. **Identify Lock Contention or Stalled Transactions**:
   ```sql
   -- Check for transactions blocking refresh_tokens or users FOR UPDATE
   SELECT blocked_locks.pid     AS blocked_pid,
          blocking_locks.pid    AS blocking_pid,
          blocked_activity.query    AS blocked_statement,
          blocking_activity.query   AS current_statement_in_blocking_process
   FROM  pg_catalog.pg_locks         blocked_locks
   JOIN pg_catalog.pg_stat_activity blocked_activity ON blocked_activity.pid = blocked_locks.pid
   JOIN pg_catalog.pg_locks         blocking_locks
       ON blocking_locks.locktype = blocked_locks.locktype
       AND blocking_locks.database IS NOT DISTINCT FROM blocked_locks.database
       AND blocking_locks.relation IS NOT DISTINCT FROM blocked_locks.relation
       AND blocking_locks.page IS NOT DISTINCT FROM blocked_locks.page
       AND blocking_locks.tuple IS NOT DISTINCT FROM blocked_locks.tuple
       AND blocking_locks.virtualxid IS NOT DISTINCT FROM blocked_locks.virtualxid
       AND blocking_locks.transactionid IS NOT DISTINCT FROM blocked_locks.transactionid
       AND blocking_locks.classid IS NOT DISTINCT FROM blocked_locks.classid
       AND blocking_locks.objid IS NOT DISTINCT FROM blocked_locks.objid
       AND blocking_locks.objsubid IS NOT DISTINCT FROM blocked_locks.objsubid
       AND blocking_locks.pid != blocked_locks.pid
   JOIN pg_catalog.pg_stat_activity blocking_activity ON blocking_activity.pid = blocking_locks.pid
   WHERE NOT blocked_locks.granted;
   ```
4. **Inspect Application Pool Logs**:
   ```bash
   journalctl -u has-auth-api -n 100 --no-pager | grep -E "sqlx|pool timed out|database error"
   ```

## 4. Mitigation (Immediate Action)
1. **Promote Standby Database (Failover)**:
   If the primary node is unreachable or experiencing hardware failure:
   - For Patroni / Cloud Native PG:
     ```bash
     patronictl -c /etc/patroni/has_auth.yml failover
     ```
   - For AWS RDS / Aurora:
     ```bash
     aws rds reboot-db-instance --db-instance-identifier has-auth-primary --force-failover
     ```
2. **Terminate Long-Running Blocking Queries**:
   If transactions are holding row locks on `refresh_tokens`:
   ```sql
   SELECT pg_terminate_backend(pid)
   FROM pg_stat_activity
   WHERE query LIKE '%FOR UPDATE%'
     AND state_change < NOW() - INTERVAL '30 seconds'
     AND pid <> pg_backend_pid();
   ```
3. **Restart Auth API Instances**:
   If SQLx connection pools entered an unrecoverable broken pipe state during DNS switch:
   ```bash
   systemctl restart has-auth-api
   ```

## 5. Root Cause Categories
- **Database Hardware / Host Crash**: Primary DB VM unreachable or kernel out of memory.
- **Deadlock / Long-Held Row Lock**: Concurrent refresh transactions or bulk admin updates blocking rows indefinitely.
- **Connection Leak**: Unclosed connections in backend worker or external batch job consuming max_connections.

## 6. Rollback Procedure
If failover promotion fails or the promoted standby is read-only:
1. Re-verify replica promotion status:
   ```sql
   SELECT pg_is_in_recovery(); -- Must return 'f' (false) on promoted primary
   ```
2. Re-point `DATABASE_URL` environment variable to the verified writable instance.
3. Restart auth service to reset SQLx connection pools.
4. Verify error rate returns to zero:
   ```bash
   curl -s http://localhost:8080/metrics | grep 'auth_middleware_rejection_total{reason="internal_error"}'
   ```

## 7. Related Runbooks
- `docs/runbooks/redis-outage.md`
- `docs/runbooks/mass-logout-incident.md`
