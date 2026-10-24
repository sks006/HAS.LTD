# Runbook: Signing Key Rotation & Verification

## 1. Alert Name and Short Description
- **Alert Name**: `AuthHighInvalidTokenRate`
- **Short Description**: Elevated rate of HTTP 401 rejections with `reason="invalid_token"` in auth middleware. Indicates an RSA public/private key mismatch, expired JWT keys, key corruption, or an unsynchronized key rotation deployment.

## 2. Symptoms
- **User Impact**: Users with valid sessions are abruptly rejected across all protected endpoints with HTTP 401 Unauthorized (`error: { code: "invalid_token", message: "Invalid or expired access token" }`).
- **Dashboard Indications**:
  - `Auth Health`: Sharp spike in "Auth Middleware Latency" and rejection errors.
  - `Auth Security`: "Middleware Rejections by Reason" panel shows `invalid_token` exceeding 100x baseline.
  - Spike in `auth_middleware_rejection_total{reason="invalid_token"}`.

## 3. Diagnosis Steps
1. **Check Application Logs**:
   ```bash
   # Search logs for JWT verification failures and key loading errors
   journalctl -u has-auth-api -n 200 --no-pager | grep -E "jwt|verify_access_token|invalid_token|public_key"
   ```
2. **Inspect Current JWT Public Key in Service Memory & File System**:
   ```bash
   # Verify the active public key fingerprint matches deployment secrets
   ssh deploy@auth-node-01 "sha256sum /etc/has-auth/keys/jwt_rsa_public.pem"
   ssh deploy@auth-node-02 "sha256sum /etc/has-auth/keys/jwt_rsa_public.pem"
   ```
3. **Verify Token Claims from Inbound Request**:
   ```bash
   # Capture a sample bearer token from headers (ensure redaction layer active)
   # Check kid or token signature validity using local crypto tool:
   cargo run --bin test_key_verification -- --pub-key /etc/has-auth/keys/jwt_rsa_public.pem --token "$BEARER_TOKEN"
   ```
4. **Inspect Node Version Divergence**:
   ```bash
   # Check if canary or partial rolling deployment left nodes with mismatched signing keys
   ansible all -m shell -a "sha256sum /etc/has-auth/keys/jwt_rsa.pem"
   ```

## 4. Mitigation (Immediate Action)
1. **Synchronize Key Distribution**:
   If a cluster node has an outdated or corrupted key pair:
   ```bash
   # Resync the current authoritative key pair from HashiCorp Vault / Secret Manager
   vault kv get -field=private_key secret/has/auth/jwt > /etc/has-auth/keys/jwt_rsa.pem
   vault kv get -field=public_key secret/has/auth/jwt > /etc/has-auth/keys/jwt_rsa_public.pem
   chmod 600 /etc/has-auth/keys/jwt_rsa.pem
   systemctl reload has-auth-api
   ```
2. **Dual-Key Verification Window (Zero-Downtime Rotation)**:
   If rotating to a new key pair:
   - Ensure new public keys are deployed to all API instances 15 minutes before the new private key is activated.
   - Access tokens have a 15-minute TTL (`exp = iat + 900`). Wait 15 minutes for existing tokens to naturally expire or refresh.
3. **Trigger Client Refresh**:
   Clients encountering `invalid_token` will invoke `/auth/refresh` using their `__Secure-has_refresh` cookie to receive a token signed by the active private key.

## 5. Root Cause Categories
- **Deploy Asynchrony**: API nodes updated with new signing key before verification keys were distributed to all edge verifiers.
- **Key Expiration / Deprecation**: Scheduled rotation script rotated private key without maintaining previous public key in verification ring.
- **Corrupted Secrets Mount**: Secret agent or CSI volume driver mounted an empty or malformed PEM file.

## 6. Rollback Procedure
If key synchronization fails or signing keys cannot be validated:
1. Revert to the previously known good key pair from secret backup:
   ```bash
   cp /etc/has-auth/keys/jwt_rsa.pem.bak /etc/has-auth/keys/jwt_rsa.pem
   cp /etc/has-auth/keys/jwt_rsa_public.pem.bak /etc/has-auth/keys/jwt_rsa_public.pem
   systemctl restart has-auth-api
   ```
2. Confirm verification error rate drops:
   ```bash
   curl -s http://localhost:8080/metrics | grep 'auth_middleware_rejection_total{reason="invalid_token"}'
   ```

## 7. Related Runbooks
- `docs/runbooks/mass-logout-incident.md`
- `docs/runbooks/suspected-account-breach.md`
