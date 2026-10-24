# Runbook: Redis Outage & Grace Cache Degradation

## 1. Alert Name and Short Description
- **Alert Name**: `AuthArgon2QueueTimeouts` / `AuthEmailDispatchFailures`
- **Short Description**: Redis instance/cluster is unreachable, timing out, or out of memory. Affects rate limiting, refresh single-flight locks, 15-second grace window cache, and auth_version caching.

## 2. Symptoms
- **User Impact**:
  - `/auth/refresh` retries within the 15-second grace window fail closed with HTTP 401 `token_invalid` instead of returning the cached successor (prevents false family revoking per Phase 5 Decision b).
  - Rate limits fail open or degrade gracefully depending on endpoint design.
  - Latency spikes on all authenticated endpoints as auth middleware falls back to querying PostgreSQL directly for `auth_version` and `status`.
- **Dashboard Indications**:
  - `Auth Health`: "DB Query Rate" surges 5x-10x due to cache miss fallbacks.
  - "Argon2 Queue Timeout Rate" spikes if background tasks stall.
  - Increase in `auth_email_dispatch_failure_total`.

## 3. Diagnosis Steps
1. **Check Redis Connectivity & Process Health**:
   ```bash
   # Ping Redis endpoint
   redis-cli -h $REDIS_HOST -p $REDIS_PORT PING
   # Check response time and latency
   redis-cli -h $REDIS_HOST -p $REDIS_PORT --latency
   ```
2. **Inspect Memory and Eviction Metrics**:
   ```bash
   redis-cli -h $REDIS_HOST -p $REDIS_PORT INFO memory
   redis-cli -h $REDIS_HOST -p $REDIS_PORT INFO clients
   ```
3. **Inspect Application Error Logs**:
   ```bash
   journalctl -u has-auth-api -n 100 --no-pager | grep -E "Redis error|grace cache|refresh lock"
   ```
4. **Assess PostgreSQL Load Impact**:
   ```sql
   -- Check if database connection pool is approaching max limit due to cache bypass
   SELECT count(*) AS total_conns, state FROM pg_stat_activity GROUP BY state;
   ```

## 4. Mitigation (Immediate Action)
1. **Restart or Fail Over Redis**:
   - If using Redis Sentinel / AWS ElastiCache / Redis Cluster:
     ```bash
     # Trigger manual failover to healthy replica
     redis-cli -h $SENTINEL_HOST -p 26379 SENTINEL FAILOVER mymaster
     ```
   - If standalone service:
     ```bash
     sudo systemctl restart redis-server
     ```
2. **Clear Stale Refresh Locks and Memory**:
   If Redis is OOM (`OOM command not allowed when used memory > 'maxmemory'`):
   ```bash
   # Switch eviction policy to volatile-lru if not configured
   redis-cli -h $REDIS_HOST -p $REDIS_PORT CONFIG SET maxmemory-policy allkeys-lru
   ```
3. **Scale PostgreSQL Connection Pool**:
   If database queries surge while Redis is recovering:
   - Temporarily increase pool size in environment: `DATABASE_MAX_CONNECTIONS=100`.
   - Reload auth service: `systemctl reload has-auth-api`.

## 5. Root Cause Categories
- **Memory Exhaustion (OOM)**: Unbounded key growth without TTLs or missing LRU eviction policy.
- **Network Partition**: Firewall rule, VPC peering disruption, or DNS failure resolving Redis hostname.
- **Client Connection Exhaustion**: Connection leak in async connection pool filling Redis maxclients (default 10,000).

## 6. Rollback Procedure
Once Redis connection is re-established:
1. Verify ping returns `PONG`.
2. Confirm API logs report successful cache connections.
3. Monitor `auth_db_query_total` on "Auth Health" dashboard to verify query rates return to baseline.
4. Verify `/auth/refresh` succeeds with single-flight mutex:
   ```bash
   curl -s -X POST http://localhost:8080/auth/refresh \
     -H "Cookie: __Secure-has_refresh=$TEST_TOKEN"
   ```

## 7. Related Runbooks
- `docs/runbooks/postgres-failover.md`
- `docs/runbooks/suspected-account-breach.md`
