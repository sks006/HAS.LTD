# Cryptographic Primitives Decision Register (Phase 2)

This document formalizes architectural and cryptographic decisions for Phase 2 of the **HAS.LTD** platform authentication and security architecture.

---

## 1. Argon2id Parameters

### Specification
* **Algorithm:** Argon2id (RFC 9106 hybrid mode resisting both side-channel and GPU/ASIC time-memory trade-off attacks).
* **Memory Cost ($m$):** 19,456 KiB (19 MiB).
* **Time Cost ($t$):** 2 iterations.
* **Parallelism ($p$):** 1 lane (1 OS thread).
* **Salt Length:** 16 bytes (128 bits, sourced from `rand::rngs::OsRng`).
* **Output / Tag Length:** 32 bytes (256 bits).

### Justification Against OWASP 2024 Minimums & Deployment Context
* **OWASP 2024 Compliance:** The OWASP Password Storage Cheat Sheet (2024) specifies three standard configurations for Argon2id:
  1. *Container/Web server standard:* $m = 19456\text{ KiB}$ ($19\text{ MiB}$), $t = 2$, $p = 1$.
  2. *Alternative low-memory:* $m = 47104\text{ KiB}$ ($46\text{ MiB}$), $t = 1$, $p = 1$.
  3. *High-security / dedicated auth worker:* $m = 65536\text{ KiB}$ ($64\text{ MiB}$), $t = 1$, $p = 4$.
* **Deployment Context (Single-Region Docker, unknown CPU & concurrency):**
  In containerized Docker runtimes (e.g., 2 vCPU, 2–4 GiB RAM), configuring $p = 4$ or $m = 64\text{ MiB}$ per verification introduces catastrophic thread contention and memory starvation under concurrent login bursts. Setting $p = 1$ ensures that a single hashing operation does not attempt multi-threaded work inside a CPU-constrained cgroup, while $m = 19456\text{ KiB}$ and $t = 2$ provides robust ASIC/GPU resistance with an average CPU cost of ~45–60 ms on modern x86_64/AArch64 cores.
* **Why Better:** Strict compliance with OWASP 2024; minimal lock contention; deterministic memory footprint of ~19.5 MiB per hashing instance.
* **Why Not:** Higher work factors ($m=64\text{ MiB}$) would give larger offline cracking resistance if the database were stolen, but would drastically decrease container throughput.
* **Alternative Rejected:**
  - *Argon2i / Argon2d:* Rejected. Argon2d is vulnerable to side-channel cache-timing attacks; Argon2i is less resistant to GPU memory trade-offs. RFC 9106 recommends Argon2id for password hashing.
  - *Bcrypt / PBKDF2:* Rejected. Bcrypt has 72-byte password truncation limits and lack of memory-hardness against modern FPGA/ASIC clusters. PBKDF2 has zero memory hardness.
  - *$p = 4$ (multi-threaded per hash):* Causes severe cgroup CPU throttling in standard 1–2 vCPU containers.

### DoS Ceiling Analysis & Queue Dynamics
* **Memory Ceiling:**
  - In a 2 GiB Docker container, allocating ~512 MiB to OS, Tokio runtime, Redis, and PostgreSQL pool leaves ~1,536 MiB for concurrent memory-intensive tasks.
  - Maximum concurrent in-flight Argon2 operations: $1536\text{ MiB} / 19.5\text{ MiB} \approx 78$ operations before Linux OOM killer invocation.
* **CPU Saturation Ceiling:**
  - On 2 vCPUs, assuming 50 ms per verification on 1 core: each core handles 20 verifications/second.
  - Total sustained throughput: $2 \times 20 = 40\text{ verifications/second}$.
* **Queue Dynamics:**
  - If incoming login traffic exceeds 40 req/s, tasks buffer in the executor queue. Unbounded task submission causes queue explosion, latency spikes (>10s), socket timeouts, and cascading retries.
* **`spawn_blocking` Pool Sizing Implication:**
  - Tokio's default `max_blocking_threads` is **512**.
  - If 512 Argon2 tasks run concurrently: $512 \times 19.5\text{ MiB} \approx 10.0\text{ GiB}$ RAM required.
  - In any container with less than 10 GiB RAM, Tokio's default blocking thread pool configuration will cause an immediate catastrophic OOM crash under load.
  - **Remediation:** Application and crypto layer must isolate password verifications behind a concurrency limiter (Tokio `Semaphore` with permits bounded to $2 \times \text{vCPU}$, e.g. 4–8 permits) or configure Tokio runtime `max_blocking_threads(32)`.

### Reversibility, Threat-Model, GDPR, and Cost
* **Reversibility:** High. Passwords can be rehashed on successful login via `needs_rehash()`.
* **Threat-Model Impact:** Defends against offline cracking of leaked credential tables; immune to GPU/rainbow-table acceleration.
* **GDPR Impact:** Article 32 ("Security of processing") state-of-the-art compliance for credential confidentiality.
* **Operational Cost:** ~50 ms CPU time per login; memory usage capped by concurrency bounds.

---

## 2. SHA-256 vs Argon2 for Refresh Tokens

### Specification
* **Generation:** 32 cryptographically secure bytes (256 bits) from `rand::rngs::OsRng`.
* **Client Token:** Plaintext encoded as unpadded URL-safe Base64 (`base64url`, 43 characters).
* **Storage Hash:** SHA-256 computed directly over the raw 32 bytes (64-character lowercase hex string).

### Justification: High-Entropy Secrets vs Passwords
* **Entropy Analysis:** The refresh token has 256 bits of true cryptographic entropy ($2^{256} \approx 1.15 \times 10^{77}$ possible combinations).
* **Work-Factor Requirement:** Argon2's memory hardness and CPU iterations exist solely to amplify the search cost of low-entropy secrets (human passwords with 30–40 bits of entropy) against offline dictionary and brute-force attacks.
* **Why SHA-256 is Optimal:**
  - SHA-256 provides 256-bit second-preimage and preimage resistance.
  - Brute-forcing a 256-bit random key from its SHA-256 digest requires $2^{256}$ operations, which violates thermodynamic feasibility regardless of how fast SHA-256 is.
  - If the database is compromised, an attacker gaining access to `refresh_tokens.token_hash` cannot derive the plaintext token without searching a $2^{256}$ space.
* **Why this does NOT Generalize to Passwords:** Passwords have low entropy and can be guessed via wordlists and masks. An attacker computing billions of SHA-256 hashes per second on a GPU would crack a password hash in seconds.
* **Why Not Argon2 for Tokens:** Applying Argon2 to a 256-bit random token wastes 50 ms CPU and 19.5 MiB RAM per token refresh with zero security benefit, opening an unnecessary DoS vector on session renewal.
* **Alternative Rejected:** Storing plaintext tokens (rejected: DB read compromise grants full session hijacking); SHA-1 / MD5 (rejected: broken collision resistance, deprecated).
* **Reversibility:** High. Hash algorithm can be migrated on token issuance.
* **Threat-Model Impact:** Complete protection against database dump privilege escalation via token replay.
* **GDPR Impact:** Eliminates persistent plaintext authorization credentials in storage.
* **Operational Cost:** Negligible (< 1 microsecond CPU, 0 MB heap allocation).

---

## 3. SHA-256 vs Argon2 for Email OTP (6 Digits, 10-Minute TTL)

### Specification
* **Format:** 6-digit numeric token (`000000`–`999999`).
* **Validity Period:** 10 minutes.
* **Storage:** SHA-256 digest of `(salt || otp)` or SHA-256 hex digest, combined with strict online attempt tracking (max 3–5 attempts, then invalidate).

### Tradeoff Analysis & Decision
* **Entropy Analysis:** A 6-digit OTP contains exactly $10^6$ combinations ($\approx 19.93$ bits of entropy).
* **Offline Cracking Comparison:**
  - *SHA-256:* A single modern consumer GPU (Nvidia RTX 4090) computes ~30 billion SHA-256 hashes/sec. Computing $10^6$ hashes takes **33 microseconds**.
  - *Argon2id ($m=19456, t=2, p=1$):* Computing $10^6$ Argon2id hashes at 50 ms/hash takes **50,000 seconds (~13.8 CPU hours)**. Even on a multi-GPU cracking cluster, cracking time exceeds the 10-minute expiry window.
* **Decision: SHA-256 + Aggressive Online Rate Limiting + User-Scoped Salt**
  - *Online Defense:* Attacking a live system requires guessing across $10^6$ combinations. Gating the verification endpoint at 3 failed attempts yields a maximum successful guess probability of $3 / 1,000,000 = 0.0003\%$ per 10-minute cycle.
  - *Residual Risk:* If the database is exfiltrated in real time *and* an attacker intercepts the specific target user's OTP generation timestamp, the attacker has a 10-minute window to dump the hash, crack the SHA-256 digest in microseconds, and use it before the TTL expires or the legitimate user completes login.
  - *Why not Argon2 for OTP:* Argon2 on OTP verification would expose the email verification endpoint to CPU exhaustion attacks (an attacker triggering repeated OTP verifications to starve the API). The 10-minute ephemeral lifespan and 3-attempt kill-switch mitigate the online threat.

### Reversibility, Threat-Model, GDPR, and Cost
* **Reversibility:** High.
* **Threat-Model Impact:** Online brute-force eliminated by attempt caps; residual risk confined to real-time (<10 min) DB exfiltration.
* **GDPR Impact:** Minimal. OTPs expire and are deleted within 10 minutes.
* **Operational Cost:** < 2 microseconds per verification.

---

## 4. RS256 Key Size: 2048 vs 4096

### Specification
* **Algorithm:** RS256 (RSASSA-PKCS1-v1_5 with SHA-256).
* **Key Size:** **2048 bits**.
* **Key Format:** PKCS#8 PEM for private keys (`0600`), SPKI PEM for public keys (`0644`).

### Comparative Tradeoff
| Metric | RSA-2048 | RSA-4096 |
|---|---|---|
| **Security Equivalent** | 112 bits (NIST/BSI compliant to 2030+) | 128 bits |
| **Signature Size** | 256 bytes (342 chars base64url) | 512 bytes (684 chars base64url) |
| **Full JWT Header+Claims+Sig** | ~600–750 bytes | ~1,100–1,300 bytes |
| **Sign Latency (Single Core)** | ~1.1 ms | ~7.8 ms (~7x slower) |
| **Verify Latency ($e=65537$)** | ~0.06 ms | ~0.18 ms |
| **HTTP Header Bloat** | Standard | High (doubled header overhead on every internal/API request) |

### Decision & Justification
* **Why RSA-2048:**
  - Access tokens have short lifespans (15 minutes) and keys are rotated regularly.
  - RSA-2048 provides cryptographic safety well beyond 2030 according to NIST SP 800-57 Part 1 Rev. 5 and BSI TR-02102-1.
  - RSA-4096 creates a 7x signing latency penalty and doubles signature payload in HTTP `Authorization` headers.
* **Alternative Rejected:** RSA-4096 (excessive latency and size for short-lived access tokens); Ed25519 (ideal, but prompt specifically mandates RS256).
* **Reversibility:** High. New keypairs can be generated at any key size during standard rotation.
* **Operational Cost:** ~1 ms per token issuance, ~0.06 ms per verification.

---

## 5. JWT Key Identification (`kid`) Strategy

### Specification & Rules
* **Active Key Count:** At least 2 active keypairs maintained in `.secrets/jwt/`:
  - 1 active signing key (the key with the highest numeric/lexicographic `kid`).
  - 1 or more verification-only keys (older keys retained for validating in-flight tokens within the 15-minute token TTL window).
* **Key Selection on Sign:**
  - `JwtKeys` scans loaded private keys and deterministically selects the private key corresponding to the **highest `kid`**.
* **Key Selection on Verify:**
  - Parse the unverified JWT header via `jsonwebtoken::decode_header`.
  - Extract the `kid` header field.
  - Select the public `DecodingKey` corresponding to `kid` from the in-memory map.
* **Unknown or Missing `kid` Behavior:**
  - Missing `kid`: **Immediate rejection** (`CryptoError::MissingKeyId`).
  - Unknown `kid`: **Immediate rejection** (`CryptoError::UnknownKeyId`).
  - Tokens signed with non-RS256 algorithms (such as `alg=none`, `HS256`, or symmetric HMAC with RSA public key): **Strictly rejected**.
* **Why Better:** Prevents key confusion attacks (CVE-2015-9235, CVE-2016-5431); enables zero-downtime key rotation.
* **Alternative Rejected:** Single static key (downtime during rotation); attempting fallback verification across all keys (opens timing leaks and CPU DoS).
* **Reversibility:** Fully configurable via filesystem key directory.
* **Threat-Model Impact:** Prevents key confusion and algorithm downgrade attacks.

---

## 6. TOTP Algorithm and Parameters

### Specification
* **HMAC Algorithm:** **HMAC-SHA1** (RFC 6238 / RFC 4226).
* **Digit Count:** **6 digits**.
* **Time Step (Period):** **30 seconds**.
* **Skew Window:** **$\pm 1$ step** (tolerance: $[-30\text{s}, +30\text{s}]$), accepting current window $t$, $t-1$, and $t+1$. Total 90-second valid window. Steps $\le -2$ and $\ge +2$ are rejected.
* **Recovery Codes:**
  - Count: **10 codes**.
  - Length: **10 characters**.
  - Alphabet: Unambiguous alphanumeric set: `23456789ABCDEFGHJKLMNPQRSTUVWXYZ` (excludes visually confusing `0`, `O`, `1`, `I`, `l`).
  - Hashing: **SHA-256 hex digest** with constant-time equality check (`subtle::ConstantTimeEq`).
  - Single-Use Enforcement: Verification function returns the matched index so the caller atomically marks or deletes the specific code from the database.

### Justification Against RFC 6238 and Client Interoperability
* **HMAC-SHA1 vs SHA-256:**
  - While RFC 6238 allows HMAC-SHA256 and HMAC-SHA512, client support is fragmented. Google Authenticator (iOS/Android) completely ignores SHA-256 parameters in `otpauth://` URIs and defaults to SHA-1, causing verification failures for users. 1Password and Authy support SHA-1 seamlessly.
  - *Cryptographic Safety:* SHA-1 collision vulnerabilities (SHAttered) apply to hash collision, not HMAC. HMAC-SHA1 is a pseudo-random function (PRF) and remains secure for TOTP.
* **6 vs 8 Digits:** Google Authenticator does not reliably support 8-digit codes. 6 digits provides optimal UX while maintaining 1-in-a-million resistance per 30-second window.
* **Recovery Code Hashing: SHA-256 vs Argon2:**
  - Each 10-character code selected from a 32-character alphabet has $32^{10} = 2^{50} \approx 1.12 \times 10^{15}$ combinations (~50 bits entropy).
  - Storing 10 recovery codes per user hashed with SHA-256 avoids consuming 500 ms of CPU and 195 MiB RAM during recovery code generation or verification. With constant-time checking and 50 bits of entropy per code, SHA-256 provides robust security.

---

## 7. Encryption of TOTP Secret at Rest

### Specification
* **Cipher:** **AES-256-GCM** (Authenticated Encryption with Associated Data - AEAD).
* **Key Size:** 32 bytes (256 bits).
* **Nonce:** 12-byte cryptographically secure random nonce generated from `rand::rngs::OsRng` per encryption call.
* **Ciphertext Layout:** Prepend 12-byte nonce directly to ciphertext: `[12-byte nonce][encrypted_bytes_with_16_byte_tag]`.

### Justification: Why AEAD?
* Unauthenticated ciphers (such as AES-CBC or AES-CTR) are vulnerable to bit-flipping and padding oracle attacks.
* AES-256-GCM provides confidentiality and cryptographic integrity verification in a single pass. Any modification to the ciphertext or nonce causes decryption to fail immediately.

### Key Management, Rotation & Disaster Recovery
* **Location:** Key loaded from `.secrets/totp.key` (file permissions `0600`) or `TOTP_ENCRYPTION_KEY` environment variable (base64 or hex). In cloud production, injected via Docker secret / AWS KMS / HashiCorp Vault.
* **Rotation Strategy:** Decrypt existing records using old key and re-encrypt with new key via an asynchronous maintenance job.
* **Key Loss Consequences & Mitigation:**
  - AES-256-GCM is mathematically non-recoverable without the key. If the key is permanently lost, stored TOTP secrets cannot be decrypted.
  - *Mitigation:* Recovery codes are stored as independent SHA-256 hashes, not encrypted with `totp.key`. Users can authenticate using their recovery codes and re-bind a new TOTP secret.

---

## 8. Zeroization Boundaries

### Specification
* **Derivation:** Types holding sensitive secret material derive `zeroize::Zeroize` and `zeroize::ZeroizeOnDrop`.
* **Zeroized Structs:**
  - `TotpSecret(Vec<u8>)`: Encrypted and decrypted TOTP secret buffers.
  - `Zeroizing<String>` / temporary password buffers during hashing and verification.
  - Master TOTP key buffer (`[u8; 32]`).
  - RSA private key PEM memory buffers before ingestion by `jsonwebtoken`.
  - Plaintext recovery code buffers during generation.

### Boundary Between Secret in Memory and Cleared
* Plaintext secret strings and byte slices reside only in scoped zeroizing buffers.
* Once the cryptographic operation (Argon2 verification, AES-GCM decryption/encryption, or TOTP generation) finishes, the buffer goes out of scope and the memory is overwritten with zeros using volatile writes to prevent compiler dead-store optimization.
* **Why Not Zeroize Everything?** Public keys, UUIDs, non-sensitive JWT claims (`sub`, `iss`, `role`), and one-way SHA-256 hashes do not represent secret keys. Zeroizing all non-sensitive memory imposes unnecessary runtime overhead and breaks standard Rust `Copy`/`Clone` semantics.

---

## 9. `.secrets/` Layout and Loading

### Directory Structure & Permissions
```
.secrets/                       [0700: rwx------]
├── jwt/                        [0700: rwx------]
│   ├── 1.pem                   [0600: rw-------] (Active RS256 Private Key, PKCS#8)
│   ├── 1.pub.pem               [0644: rw-r--r--] (RS256 Public Key, SPKI)
│   ├── 2.pem                   [0600: rw-------] (Rotation-in RS256 Private Key, PKCS#8)
│   └── 2.pub.pem               [0644: rw-r--r--] (RS256 Public Key, SPKI)
└── totp.key                    [0600: rw-------] (32 random bytes master key)
```

### Automation & Loading Lifecycle
1. **`.gitignore` Pre-Creation:** `.secrets/` MUST be present in `.gitignore` before directory generation.
2. **Generation Script:** `scripts/init_secrets.sh` generates keys via OpenSSL, verifies `.secrets/` does not already exist, sets exact POSIX permissions (`chmod 0700` and `chmod 0600`), and logs key fingerprints.
3. **Loader Precedence:**
   `Environment Variable` (`APP_CRYPTO__*`) > `Configuration File` (`config.toml`) > `Default Paths` (`.secrets/jwt`, `.secrets/totp.key`).
4. **Missing Key at Startup:** Application fails fast and halts bootstrap with a descriptive fatal error if `.secrets/jwt` is empty or `totp.key` is missing or not exactly 32 bytes. No default fallback secrets are permitted.

---

## 10. Phase 2 Amendments (Phase 3 Review)

### [FIX-H7] JWT Claims `scope` Field Amendment
* **Specification:** Added `scope: String` to `Claims` struct in `backend/crypto/src/jwt.rs`.
* **Value Constraints:**
  - Standard User/Admin Session Tokens: `scope == "access"`.
  - Ephemeral Multi-Factor Authentication Challenges: `scope == "mfa_pending"`.
* **Middleware Invariant:** Authentication guard middleware MUST reject any token where `claims.scope != "access"` with HTTP 401 `invalid_token`. Tokens with `mfa_pending` are strictly rejected on standard API routes.
* **Why Better:** Cryptographic intent isolation between fully-authenticated user sessions and partial-state MFA challenges; prevents elevation of privilege via unauthenticated use of ephemeral MFA tokens against standard API routes.
* **Reversibility:** High.
* **Threat-Model Impact:** Eliminates challenge replay and privilege bypass.
* **GDPR / Operational Cost:** Negligible (< 12 bytes additional JWT payload size).

