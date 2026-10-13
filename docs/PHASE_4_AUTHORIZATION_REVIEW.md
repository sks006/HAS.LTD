# Phase 4 — Authorization Layer Review & Decision Register

**Status:** Under Implementation  
**Date:** 2026-10-14  
**Author:** Antigravity Pairing Assistant  
**Inputs:** Approved Phase 1 (Schema & Domain), Approved Phase 2 (Crypto Primitives), Approved Phase 3 (Design Doc & Flow Corrections C1–M5)

---

## 1. Decision Register

Each decision addresses: **Why Better**, **Why Not Alternative**, **Alternative Rejected**, **Reversibility**, **Threat Model Impact**, **GDPR Impact**, and **Operational Cost**.

---

### a) Middleware Architecture

* **Decision:** Per-route extractors (`AuthUser`, `RequireCustomer`, `RequireModerator`, `RequireAdmin`) underpinned by a shared core authentication helper (`authenticate_request`) that caches `Arc<AuthenticatedUser>` in `parts.extensions`.
* **Why Better:**
  1. *Axum 0.8 Extractor Semantics:* Axum 0.8 uses `FromRequestParts` with `FromRef<AppState>` for typed dependency extraction. Declaring role constraints directly in handler parameter signatures (`user: RequireAdmin`) makes authorization requirements explicit, self-documenting, and compile-time verified at handler definition.
  2. *Shared Extension Caching:* The core helper evaluates authentication once per request and memoizes `Arc<AuthenticatedUser>` in `Request::extensions`. Downstream extractors reuse the parsed token without redundant signature verification or Redis roundtrips.
  3. *Unit Testability:* Extractors can be invoked in unit tests by constructing synthetic `Parts` and `AppState` without launching a Tower service or network listener.
  4. *Error Uniformity:* The rejection type `AuthRejection` implements `IntoResponse`, reliably serializing the frozen `{ "error": { "code": "...", "message": "..." } }` contract.
* **Why Not Alternative (Single Tower Layer with Role Config):** A single generic Tower layer requires dynamic path-matching tables or route-metadata injection, separating authorization policy from handler code and introducing routing desynchronization risks.
* **Alternative Rejected:** Pure Tower route-layer middleware (`.layer(auth_layer(Role::Admin))`).
* **Reversibility:** High. Extractors wrap standard Axum `FromRequestParts`; migrating to a Tower layer would simply invoke the same `authenticate_request` logic.
* **Threat-Model Impact:** Eliminates route-guard detachment risks where a developer adds a handler but forgets to apply the middleware wrapper.
* **GDPR Impact:** Zero. No PII is logged or cached outside existing access-token boundaries.
* **Operational Cost / p99 Cost:**
  - RS256 Ring verification: ~0.08 ms (80 µs).
  - Redis multiplexed `auth_version` + `user_status` lookup: ~0.20–0.50 ms over local network.
  - Zero database queries on warm cache.
  - Total p99 valid request overhead: **< 0.60 ms**.

---

### b) Where Each Check Runs

* **Decision:** Strict pipeline partitioning between extractor and handler.
  1. *Signature Verification:* Extractor core (`authenticate_request`), executed prior to any network I/O or database access.
  2. *exp/nbf/iss/aud Validation:* Extractor core via `JwtKeys::verify_access_token`.
  3. *Scope Check (`scope == "access"`):* Extractor core immediately following signature verification; rejects `scope == "mfa_pending"` with HTTP 401 `invalid_token`.
  4. *`auth_version` Invalidation:* Extractor core via Redis `auth_version:{user_id}`, falling back to DB `SELECT auth_version, status FROM users WHERE id = $1` on cache miss.
  5. *Status Check (`status == ACTIVE`):* Extractor core via Redis `user_status:{user_id}` (or same DB query as `auth_version`).
  6. *Role Verification:* Specific extractor (`RequireCustomer`, `RequireModerator`, `RequireAdmin`).
* **Why Better:** Prevents resource-intensive database lookups for unauthenticated or forged requests. Ensures handlers receive fully verified domain credentials and never inspect raw JWT tokens.
* **Alternative Rejected:** Performing database lookup before signature verification (DoS risk), or delegating token checks to handlers (logic fragmentation).
* **Reversibility:** High. All validation is centralized in `backend/api/src/middleware/auth_guard.rs`.
* **Threat-Model Impact:** Protects against DoS amplification attacks by guaranteeing zero database work until cryptographic authenticity and scope are proven.
* **GDPR Impact:** No raw tokens or user credentials appear in logs or error traces.
* **Operational Cost:** Incurs zero database read load when cache is warm.

---

### c) 401 vs 403 Boundary

* **Decision:** Strict adherence to Phase 3 §1.f status code contract:
  - `401 invalid_token`: Missing Authorization header, malformed header, missing/unknown `kid`, invalid RS256 signature, `scope != "access"`.
  - `401 token_expired`: Token expired (`exp` in past) or immature (`nbf` in future).
  - `401 token_revoked`: Token `claims.auth_version != live_auth_version`.
  - `403 account_suspended`: User status is `SUSPENDED`, `DELETED`, or `PENDING`.
  - `403 permission_denied`: User's authenticated role is insufficient for route minimum role.
  - `403 email_not_verified`: Hard gate for specific handlers requiring email verification (checkout, order history).
* **Why Better:** Clear separation: 401 signifies missing or invalid authentication credentials (authentication failure); 403 signifies identity is recognized but access is prohibited (authorization failure).
* **Alternative Rejected:** Returning 404 for unauthorized administrative routes (leaks timing and creates debugging friction without genuine obscurity gain since API specifications are standard).
* **Reversibility:** Low (frozen external API contract).
* **Threat-Model Impact:** Eliminates resource existence enumeration by executing authorization checks before handler resource lookups.
* **GDPR Impact:** None.
* **Operational Cost:** Zero additional latency.

---

### d) Redis Client Choice

* **Decision:** `redis 0.24` with `redis::aio::ConnectionManager`.
* **Why Better:**
  1. *Connection Multiplexing:* `ConnectionManager` provides a lightweight, cloneable handle that multiplexes commands over a single pipelined async connection, eliminating thread contention and connection pooling overhead.
  2. *Automatic Reconnection:* Transparently reconnects in the background upon connection drops or Redis restarts.
  3. *Graceful Degradation:* Returns immediate errors when Redis is unreachable, allowing rate-limiting to fail-open and user-version lookups to fall back to PostgreSQL without stalling requests.
  4. *Zero Startup Panics:* URL parsing is non-blocking; `ConnectionManager` initialization attempts a handshake but allows the application to boot in degraded mode if Redis is temporarily offline.
* **Alternative Rejected:**
  - `deadpool-redis`: Unnecessary connection pooling overhead for Redis protocols which are natively multiplexed.
  - `fred`: Heavyweight client with complex clustering dependencies not required for single-instance or Sentinel topologies.
* **Reversibility:** Medium. Redis commands are accessed via standard `AsyncCommands`.
* **Threat-Model Impact:** Fails open on rate limiting per Phase 3 §1.h (preserving service availability) while failing closed on OTP/MFA challenges.
* **GDPR Impact:** None.
* **Operational Cost:** Minimal memory footprint (single multiplexed socket).

---

### e) AppState Shape

* **Decision:**
  ```rust
  #[derive(Clone)]
  pub struct AppState {
      pub db_pool: sqlx::PgPool,
      pub redis: Option<redis::aio::ConnectionManager>,
      pub jwt_keys: std::sync::Arc<crypto::jwt::JwtKeys>,
      pub argon2_semaphore: std::sync::Arc<tokio::sync::Semaphore>,
      pub config: std::sync::Arc<crate::config::AppConfig>,
  }
  ```
* **Why Better:**
  - `jwt_secret: Arc<String>` is removed, replacing legacy HS256 symmetric secret with asymmetric `Arc<JwtKeys>`.
  - `redis` is wrapped in `Option` to enable graceful degraded mode when Redis is unreachable at startup.
  - `argon2_semaphore` prevents server thread starvation under heavy password hashing loads.
  - `config` centralizes immutable system configuration.
* **Alternative Rejected:** Retaining `jwt_secret: Arc<String>` alongside `JwtKeys`.
* **Reversibility:** High.
* **Threat-Model Impact:** Eliminates symmetric shared secrets; RS256 private keys remain in memory only where required.
* **GDPR Impact:** None.
* **Operational Cost:** Zero allocation overhead per request.

---

### f) Semaphore Placement

* **Decision:** Field on `AppState` (`Arc<tokio::sync::Semaphore>`), wrapped by `Argon2Gate`.
* **Why Better:**
  - Enables granular unit and integration testing with custom permit counts (e.g., testing 8 concurrent requests against 4 permits).
  - Eliminates hidden global state.
  - Enforces a 2,000 ms acquire timeout per Phase 3 §1.h.
  - Emits telemetry: gauge `auth_argon2_queue_depth` and counter `auth_argon2_queue_timeout_total`.
* **Alternative Rejected:** Static `OnceLock<Semaphore>` (prevents isolated test runner execution with varying concurrency limits).
* **Reversibility:** High.
* **Threat-Model Impact:** Mitigates CPU exhaustion DoS by rejecting excess password-hashing requests with HTTP 503 `service_unavailable` after 2,000 ms.
* **GDPR Impact:** None.
* **Operational Cost:** Negligible atomic counter overhead.

---

### g) AppState Access Pattern

* **Decision:** `State<AppState>` attached via `.with_state(state)` on `Router`, extracted via `FromRef<AppState>` in extractors.
* **Why Better:** Axum 0.8 strongly enforces compile-time verification of `FromRef<AppState>`. Any extractor missing state fails compilation, preventing runtime panic bugs common with untyped `Extension<Arc<AppState>>`.
* **Alternative Rejected:** `Extension<Arc<AppState>>` per route.
* **Reversibility:** High.
* **Threat-Model Impact:** Eliminates unhandled runtime panics caused by missing state extensions.
* **GDPR Impact:** None.
* **Operational Cost:** Zero runtime overhead.

---

### h) Authorization (AuthZ) Model

* **Decision:** Hierarchical Role-Based Access Control (`Customer` < `Moderator` < `Admin`).
* **Why Better:**
  - E-commerce domain with 3 distinct roles does not require multi-tenant organizational ACLs.
  - `RequireCustomer` accepts Customer, Moderator, and Admin.
  - `RequireModerator` accepts Moderator and Admin.
  - `RequireAdmin` accepts Admin.
  - Future expansion ("moderator can refund but not delete products") can be seamlessly implemented by introducing fine-grained handler extractors (`RequirePermission(Permission::Refund)`) that check user capabilities without refactoring the underlying identity schema.
* **Alternative Rejected:** Full Attribute-Based Access Control (ABAC) or dynamic Policy Enforcement Point (PEP) evaluation engines.
* **Reversibility:** High.
* **Threat-Model Impact:** Simple, deterministic role hierarchy prevents privilege escalation and policy misconfigurations.
* **GDPR Impact:** None.
* **Operational Cost:** Integer/enum comparison (~1 ns).

---

### i) Public vs Protected Route Taxonomy

The following table defines the authoritative routing contract:

| Route | Method | Auth Required | Minimum Role | Notes |
|---|---|---|---|---|
| `/health` | GET | None | None | Public liveness probe; returns `{"status":"ok"}` |
| `/metrics` | GET | None | None | Public Prometheus metrics exposition |
| `/products` | GET | None | None | Public product catalog listing |
| `/products` | POST | Yes | ADMIN | Restricted to administrators |
| `/products/{id}` | GET | None | None | Public product detail view |
| `/products/{id}` | PUT | Yes | ADMIN | Restricted to administrators |
| `/products/{id}` | DELETE | Yes | ADMIN | Restricted to administrators |
| `/cart/sync` | POST | Yes | CUSTOMER | Customer cart synchronization |
| `/checkout` | POST | Yes | CUSTOMER | Customer checkout pipeline |
| `/orders` | GET | Yes | MODERATOR | Global order listing for staff |
| `/orders/{id}` | GET | Yes | CUSTOMER | Customer views own order; Moderator/Admin views any |
| `/orders/{id}` | DELETE | Yes | ADMIN | Order cancellation / purge |
| `/orders/{id}/status` | PUT | Yes | MODERATOR | Order fulfillment state updates |
| `/admin/inventory` | POST | Yes | ADMIN | Inventory stock adjustment |
| `/admin/metrics` | GET | Yes | ADMIN | Administrative dashboard metrics |
| `/webhooks/payment` | POST | Signature | None | Stripe/Payment HMAC verification, not user auth |
| `/auth/login` | POST | None | None | Public authentication endpoint |
| `/auth/me` | GET | Yes | CUSTOMER | Authenticated user profile |

---

### j) Testing Strategy

* **Decision:** Test environment against running PostgreSQL (`127.0.0.1:5432`) and Redis (`127.0.0.1:6379`) with dynamic test-prefixed key isolation (`test:{test_id}:*`) and unique test users (`test_{uuid}@has.ltd`).
* **Why Better:**
  - Avoids ~15s docker startup latency per test binary while retaining authentic PostgreSQL and Redis behavior.
  - Key prefixing guarantees isolation without flushing developer keys in shared development environments.
  - Integration tests generate distinct random UUIDs for all users, sessions, and credentials.
* **Alternative Rejected:** Flushing entire Redis instance between test cases (risks destroying local development data).
* **Reversibility:** High.
* **Threat-Model Impact:** Guarantees all cryptographic, caching, and database triggers execute against real runtime engines.
* **GDPR Impact:** Test fixtures use synthetic data only.
* **Operational Cost:** Fast test execution (~50 ms per test).

---

### k) Error Emission from Extractors

* **Decision:** Custom rejection type `AuthRejection` implementing `IntoResponse`, serializing strictly to the Phase 3 §1.f envelope:
  ```json
  {"error":{"code":"<string>","message":"<string>"}}
  ```
* **Why Better:** Axum 0.8 automatically routes rejection types into HTTP responses. Emitting the exact JSON structure guarantees client contracts remain unbreakable.
* **Alternative Rejected:** Generic strings or untyped error bodies.
* **Reversibility:** Low (frozen contract).
* **Threat-Model Impact:** Zero information leakage; never echoes invalid token strings, internal database error details, or stack traces.
* **GDPR Impact:** None.
* **Operational Cost:** Zero allocation overhead.

---

### l) Route Guard Implementation

* **Decision:** Type-safe Axum extractors (`RequireCustomer`, `RequireModerator`, `RequireAdmin`, `AuthUser`) on handler signatures.
* **Why Better:** Axum handler signatures fail to compile if an extractor is incorrectly referenced, eliminating runtime omissions. Handlers directly receive the type-safe `AuthenticatedUser` containing user ID, role, and session ID.
* **Alternative Rejected:** Manual authorization assertions inside handler function bodies.
* **Reversibility:** High.
* **Threat-Model Impact:** Compile-time enforced security boundaries.
* **GDPR Impact:** None.
* **Operational Cost:** Zero runtime dispatch overhead.

---

## 2. Route Taxonomy Contract

Refer to §1.i above.

---

## 3. Test Matrix & Verification Status

(Will be updated as test suite executes)

---

## 4. Compiler Warnings

* **Initial warning count in `api` crate:** 18 warnings
* **Target warning count:** 0 warnings (`cargo check --workspace` and `cargo clippy --workspace -- -D warnings`)
