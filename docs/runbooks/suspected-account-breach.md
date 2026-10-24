# Runbook: Suspected Account Breach & Replay Detection Response

## 1. Alert Name and Short Description
- **Alert Name**: `AuthRefreshReplayDetected` / `AuthHighLoginFailureRate` / `AuthHighMfaFailureRate`
- **Short Description**: Critical detection of refresh token replay (`auth.session.replay_detected`), concentrated brute-force login failures, or MFA bypass attempts. Replay implies that an adversary or network intermediary has intercepted and reused an expired/consumed refresh token.

## 2. Symptoms
- **User Impact**: Compromised user receives unexpected HTTP 401 `family_revoked` (`"Refresh token family revoked due to replay detection"`). All sessions for that user on all devices are forcefully terminated.
- **Dashboard Indications**:
  - `Auth Security`: "Refresh Replay Detected" stat panel indicates $> 0$ count.
  - "MFA Failures by Method" shows elevated failure rate for `totp` or `recovery`.
  - Spikes in `auth_refresh_replay_detected_total` and `auth_login_failures_total`.

## 3. Diagnosis Steps
1. **Identify Compromised User and Replay Context**:
   ```sql
   -- Find exact replay events in audit log
   SELECT id, user_id, ip_address, user_agent, metadata, created_at
   FROM audit_log
   WHERE event = 'auth.session.replay_detected'
   ORDER BY created_at DESC
   LIMIT 10;
   ```
2. **Inspect Token Family History for the Target User**:
   ```sql
   -- Inspect the lineage of refresh tokens in the revoked family
   SELECT id, session_id, family_id, token_hash, issued_at, expires_at, revoked_at, replaced_by_id
   FROM refresh_tokens
   WHERE family_id = (
       SELECT (metadata->>'family_id')::uuid
       FROM audit_log
       WHERE event = 'auth.session.replay_detected'
       ORDER BY created_at DESC LIMIT 1
   )
   ORDER BY issued_at ASC;
   ```
3. **Compare IP Address and User Agent Divergence**:
   Check if the successor token was issued to a different IP subnet or user agent than the replayed token request:
   ```sql
   SELECT ip_address, user_agent, created_at, event
   FROM audit_log
   WHERE user_id = $TARGET_USER_ID
     AND created_at > NOW() - INTERVAL '2 hours'
   ORDER BY created_at DESC;
   ```
4. **Determine if Client Network Retry or Actual Interception**:
   - If the replay occurred within $< 15$ seconds, check Redis grace window health (`refresh:grace:{hash}`). If Redis was down or evicted, the client may have had a benign race.
   - If the replay occurred minutes or hours later, this is a confirmed token exfiltration.

## 4. Mitigation (Immediate Action)
1. **Verify Complete Account Lockdown**:
   Confirm that the automated handler revoked all sessions and bumped `auth_version`:
   ```sql
   SELECT auth_version, status FROM users WHERE id = $TARGET_USER_ID;
   SELECT COUNT(*) FROM sessions WHERE user_id = $TARGET_USER_ID AND revoked_at IS NULL;
   ```
2. **If Confirmed Breach: Freeze Account**:
   If active exfiltration is detected, suspend the account to prevent password resets:
   ```sql
   UPDATE users SET status = 'SUSPENDED', updated_at = NOW() WHERE id = $TARGET_USER_ID;
   ```
3. **Purge Redis Cache for User**:
   ```bash
   redis-cli -h $REDIS_HOST -p $REDIS_PORT DEL "auth_version:$TARGET_USER_ID"
   redis-cli -h $REDIS_HOST -p $REDIS_PORT DEL "user_status:$TARGET_USER_ID"
   ```
4. **Notify Affected User**:
   Trigger security alert email informing user that suspicious session activity was terminated and requiring a fresh password reset and MFA re-verification.

## 5. Root Cause Categories
- **Man-In-The-Middle / Token Theft**: Refresh cookie leaked via malicious browser extension, insecure mobile transport, or compromised client device.
- **Benign Race Condition**: Mobile app with poor connection retried refresh after 15-second grace window expired or during transient Redis outage.
- **Credential Stuffing**: Distributed botnet attempting credential validation across multiple accounts.

## 6. Rollback Procedure
If verified to be a benign client race condition (e.g. mobile connectivity drop on flaky cellular network during Redis restart):
1. Reactivate the account if it was manually suspended:
   ```sql
   UPDATE users SET status = 'ACTIVE', updated_at = NOW() WHERE id = $TARGET_USER_ID;
   ```
2. User logs in normally via `/auth/login`. No manual database restoration of revoked refresh tokens is permitted.

## 7. Related Runbooks
- `docs/runbooks/mass-logout-incident.md`
- `docs/runbooks/redis-outage.md`
