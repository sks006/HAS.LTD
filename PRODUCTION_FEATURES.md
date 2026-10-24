# HAS.LTD — End-to-End Production Feature Specification

> **Document Status:** Living Document / Single Source of Truth (SSOT)  
> **Last Verified:** October 2026  
> **System Scope:** Backend (Rust / Axum), Frontend (React 18 / TypeScript / Vite), Infrastructure (PostgreSQL 16, Redis 7, S3/R2, Resend), Security & Observability (Argon2id, RS256/Ed25519, Prometheus, Grafana)

---

## 📌 Document Overview & Governance Rules

This document is the **absolute, authoritative single source of truth (SSOT)** for all product and engineering features across the **HAS.LTD** enterprise e-commerce platform.

### Priority Classifications:
* **Priority 0 (P0) — MVP (Minimum Viable Product):**  
  The foundational core required to launch the baseline e-commerce platform. Covers secure account onboarding, catalog browsing, shopping cart synchronization, transactional checkout, optimistic concurrency order management, customer order tracking, and baseline administrative panels.
* **Priority 1 (P1) — Running Production:**  
  Hardened, resilient, enterprise-grade capabilities required for live, 24/7 high-traffic commercial operation. Covers high-assurance session rotation (RTR), multi-factor authentication (MFA/TOTP), real payment gateway processing & cryptographic webhook reconciliation, distributed rate limiting, abuse mitigation, cloud object storage, production observability, alerting, automated runbooks, and containerized deployments.
* **Priority 2 (P2) — Future Implementation:**  
  Next-generation luxury commerce, global scale, and intelligence capabilities. Covers interactive 3D WebGL & Augmented Reality (AR) previews, bespoke sizing engines, multi-currency & multilingual localization, AI-powered semantic search & recommendations, multi-warehouse supply-chain routing, customer loyalty/rewards, and enterprise SSO integrations.

### Invariant Rules for Tracking:
1. Every task or pull request (PR) modifying feature boundaries **must** update this file.
2. A feature is marked as complete (`[x]`) **only when**:
   - The backend domain port, repository implementation, and API endpoint are fully merged and tested.
   - The associated database tables, constraints, and indexes are migrated.
   - The frontend state slice and user interface component are wired and verified.
   - Stubs, mocks, and simulated adapters are replaced with production-ready code (mocks are classified as `[ ]` until real integrations land).

---

## 📊 Executive Status Summary

| Priority Level | Total Tracked Features | Completed | In Progress / Planned | Readiness % |
| :--- | :---: | :---: | :---: | :---: |
| **Priority 0 (P0) — MVP** | 42 | 38 | 4 | **90.5%** |
| **Priority 1 (P1) — Running Production** | 45 | 27 | 18 | **60.0%** |
| **Priority 2 (P2) — Future Implementation** | 30 | 0 | 30 | **0.0%** |
| **Overall Platform** | **117** | **65** | **52** | **55.6%** |

---

## 🟢 Priority 0: MVP (Minimum Viable Product)

Core architectural primitives and baseline end-to-end commerce flows needed to launch the initial platform version.

### 1. Identity, Authentication & Access Control (IAM / RBAC)
- [x] **Relational Identity Schema:** PostgreSQL `users` table with Monotonic UUIDv7 IDs, normalized email indexing, and monotonic `auth_version` column.
- [x] **Credential Separation:** Dedicated `credentials` table storing Argon2id password hashes and salts isolated from user identity metadata.
- [x] **User Registration Flow:** Endpoint `POST /auth/register` returning unconditional HTTP 202 Accepted to prevent user enumeration attacks.
- [x] **Email OTP Verification:** 6-digit numeric OTP generation, Redis hashing, and verification via `POST /auth/register/verify-otp` and `POST /auth/register/resend-otp`.
- [x] **Argon2id Password Hashing:** Memory-hard cryptographic hashing (64 MB memory, 3 iterations) with constant-time `dummy_verify()` side-channel defense.
- [x] **Customer Authentication:** Endpoint `POST /auth/login` validating active accounts, credentials, and issuing asymmetric access & refresh tokens.
- [x] **Asymmetric JWT Engine:** RS256 / Ed25519 cryptographic token signing with Key ID (`kid`) support and validation of claims (`sub`, `iss`, `aud`, `exp`, `auth_version`, `role`).
- [x] **User Session Identity Check:** Protected endpoint `GET /auth/me` resolving caller profile from verified access token claims.
- [x] **Basic Session Logout:** Endpoint `POST /auth/logout` revoking session token via Redis blacklist.
- [x] **Role-Based Access Control (RBAC):** Postgres SQL `CHECK` constraint enforcement (`CUSTOMER`, `MODERATOR`, `ADMIN`) per ADR-001 with Axum typed extractors (`RequireCustomer`, `RequireModerator`, `RequireAdmin`).
- [ ] **Customer Profile Management:** Endpoints and UI for updating customer contact name, default shipping addresses, and communication preferences.

### 2. Product Catalog & Merchandising
- [x] **PostgreSQL Catalog Schema:** `categories` and `products` relational schema with UUID primary keys, unique slugs, and categorization foreign keys.
- [x] **Monetary Precision Engine:** Integer minor-units currency storage (`price_minor_units` / Decimal) preventing floating-point rounding errors.
- [x] **Public Product Exploration:** Endpoint `GET /products` with pagination (`page`, `limit`), sorting, category filtering, and season/fabric filters.
- [x] **Product Detail Retrieval:** Endpoint `GET /products/{id}` resolving product aggregate, specifications, image gallery URLs, and current stock status.
- [x] **Admin Product Creation:** Endpoint `POST /products` protected by `RequireAdmin` with slug generation and attribute serialization.
- [x] **Admin Product Mutation:** Endpoint `PUT /products/{id}` allowing administrators to modify pricing, description, fabric details, and active visibility.
- [x] **Admin Product Deletion:** Endpoint `DELETE /products/{id}` enabling catalog de-listing and inventory cleanup.
- [ ] **Multi-Variant SKU Architecture:** Full database and API support for explicit variant permutations (Size: XS–XXL, Color, Fabric weight) with independent stock numbers.

### 3. Shopping Cart & Inventory State
- [x] **Client-Side Reactive Cart Store:** Persistent Zustand cart store managing item additions, quantity modifications, price computations, and local caching.
- [x] **Cart Drawer UI:** Fly-out slide drawer displaying line items, real-time total calculation, currency formatting, and one-click checkout entry.
- [x] **Cart State Synchronization:** Endpoint `POST /cart/sync` enabling browser cart synchronization and price sanity verification.
- [x] **Inventory Reservation Engine:** Pessimistic / row-level verification preventing checkout of out-of-stock items.
- [ ] **Inventory Reservation Expiry Daemon:** Background worker automatically releasing reserved items back to available stock if checkout is abandoned within 15 minutes.

### 4. Checkout, Order Lifecycle & Concurrency (OCC)
- [x] **Order Schema & State Machine:** Relational tables `orders` and `order_items` supporting strict state transitions: `Pending` → `Reserved` → `Paid` → `Shipped` → `Cancelled`.
- [x] **Customer Checkout Ingress:** Endpoint `POST /checkout` protected by `RequireCustomer` validating addresses, line items, and idempotency headers.
- [x] **Optimistic Concurrency Control (OCC):** Monotonic `version` column on orders; concurrent worker updates fail fast on version mismatch.
- [x] **Atomic Transactional Ordering:** SQLx transactional pipeline locking inventory rows, creating order records, and writing item allocations in a single database transaction.
- [x] **Idempotency Lock Extractor:** Header `Idempotency-Key` extraction preventing duplicate order placement and double-billing on retry.
- [x] **Customer Order Tracking:** Endpoint `GET /orders/{id}` and public order lookup page allowing buyers to check fulfillment status.
- [x] **Cash-on-Delivery (COD) / Mock Settlement:** Baseline checkout completion path for initial operational and staging tests.
- [ ] **Direct Payment Gateway Integration at Checkout:** Real-time credit card processing input form on checkout page.

### 5. Frontend Storefront UI/UX
- [x] **Responsive Storefront Home:** Hero branding banner, seasonal highlights, and featured luxury abaya collections.
- [x] **Catalog Grid & Filter Sidebar:** Dynamic product filtering by category, fabric type, price range, and season.
- [x] **Product Detail View:** Image carousels, fabric descriptions, stock availability badges, and "Add to Bag" interactions.
- [x] **Checkout Flow UI:** Step-by-step shipping address form, contact validation, order review, and submission handler.
- [x] **Customer Order Tracking Page:** Status timeline tracker (`CustomerOrderPage.tsx`) rendering pending, reserved, shipped, and fulfilled stages.
- [x] **Auth Modals & Pages:** Dedicated `/login` and `/register` routes with interactive OTP digit inputs and instant feedback.
- [x] **Global Notification System:** Toaster integration (Sonner) providing instant user feedback on cart updates, auth actions, and error states.

### 6. Administrative & Operations Dashboard
- [x] **Unified Dashboard Layout:** Protected administrative layout with role-based navigation sidebar and header.
- [x] **Live Orders Management Panel:** Component `OrdersPage.tsx` displaying order table, customer info, SKU, order totals, and live state transitions.
- [x] **Order State Transition Controls:** Dropdowns to advance order states (`Pending` → `Paid` → `Shipped` → `Cancelled`) guarded by OCC version checks.
- [x] **Order Audit & Moderation Queue:** Component `ModeratorPanel.tsx` providing staff tools to review flagged orders and customer histories.
- [x] **Inventory Overview Dashboard:** Component `InventoryDashboard.tsx` displaying on-hand stock and incoming units.
- [x] **Product Creation Modal:** Component `AddProductModal.tsx` for adding new luxury garments to the live storefront without manual database access.
- [x] **Health Check Endpoint:** Public endpoint `GET /health` returning service health status.

### 7. Core Infrastructure & Tooling
- [x] **Docker Compose Scaffold:** Local container orchestration for PostgreSQL 16 and Redis 7 with persistent volume mounts.
- [x] **Automated SQLx Migrations:** Versioned SQL migration files managing schema definitions, foreign key cascades, and check constraints.
- [x] **Developer Automation Scripts:** Shell runner `./scripts/dev.sh` supporting frontend, backend, or unified full-stack startup.
- [x] **Environment Variable Configuration:** Centralized `.env` template handling database URLs, secrets, and CORS origins.

---

## 🟡 Priority 1: Running Production

Enterprise-grade requirements, defensive security layers, resilience mechanisms, external integrations, observability, and day-2 operations needed to operate at scale.

### 1. Identity Hardening, Zero-Trust Sessions & Threat Defense
- [x] **Token Family Refresh Rotation (RTR):** Single-use refresh tokens tracking `family_id`, `token_hash`, `revoked_at`, and `replaced_by_id`.
- [x] **RTR 15-Second Grace Window:** Tolerates mobile network disconnects and concurrent retry packets without prematurely invalidating active user sessions.
- [x] **Replay Attack Detection & Containment:** Automatic revocation of the entire token family upon reuse of an old refresh token, bumping `users.auth_version` and invalidating Redis session keys.
- [x] **Single-Flight Refresh Mutex:** Distributed Redis lock (`lock:refresh:{session_id}`) guaranteeing serialization of concurrent refresh requests.
- [x] **Global Mass Session Revocation:** Endpoint `POST /auth/logout-all` incrementing monotonic `auth_version` and evicting all Redis session caches instantaneously.
- [x] **Active Session Management:** Endpoints `GET /auth/sessions` and `DELETE /auth/sessions/{id}` allowing users to view and revoke specific active device sessions.
- [x] **In-Session Password Change:** Endpoint `POST /auth/password/change` validating existing password, rotating hash, issuing new session tokens, and revoking peer sessions.
- [x] **Non-Enumerating Password Reset:** Two-phase reset (`POST /auth/password/forgot` and `POST /auth/password/reset`) using single-use verification tokens without leaking user existence.
- [x] **Argon2 CPU Concurrency Gate:** Tokio semaphore-managed CPU queue (`argon2_gate.rs`) preventing HashDoS CPU exhaustion attacks under heavy load.
- [x] **Multi-Factor Authentication (TOTP):** Full RFC 6238 implementation (`POST /auth/mfa/totp/enroll`, `/verify`, `/disable`) with QR code provisioning URI generation.
- [x] **At-Rest Envelope Encryption for MFA:** AES-256-GCM symmetric encryption for TOTP secrets with system pepper and salt isolation.
- [x] **MFA Emergency Recovery Codes:** Generation of 10 single-use Argon2id-hashed recovery codes with `SELECT FOR UPDATE` atomic consumption via `POST /auth/login/recovery`.
- [x] **Scope Isolation Guard:** Token `scope` validation rejecting `mfa_pending` tokens from accessing customer or administrative routes.
- [x] **Distributed Token Bucket Rate Limiting:** Redis-backed rate limiter with exponential backoff on authentication and OTP endpoints.
- [x] **IPv6 `/64` Subnet Aggregation:** Defense against IPv6 prefix hopping by hashing client IP to `/64` subnet blocks before rate-limit evaluation.
- [x] **Defensive Security Headers:** Middleware injecting Content-Security-Policy (CSP), Strict-Transport-Security (HSTS), X-Frame-Options (`DENY`), and X-Content-Type-Options (`nosniff`).
- [x] **Strict CORS Policy:** Dynamic origin validation with credential support and explicit HTTP method/header whitelisting.
- [ ] **Adaptive Anomaly Detection:** Real-time IP geolocation and User-Agent fingerprint mismatch alerts flagging suspicious logins.
- [ ] **Automated Account Lockout:** Progressive lockout policy after consecutive failed login attempts with admin override unlock capabilities.

### 2. Commercial Payment Gateways & Webhook Integrity
- [ ] **Production Payment Gateway Adapter:** Implementation of Stripe / Adyen / Checkout.com API integration replacing mock `GatewayAdapter`.
- [ ] **3D Secure 2 (3DS) & Strong Customer Authentication (SCA):** Client challenge handling for European and GCC banking regulations.
- [ ] **Cryptographic Webhook Ingress:** Endpoint `POST /webhooks/payment` verifying HMAC-SHA256 signatures with replay timestamp validation.
- [ ] **Automated Order State Settlement:** Webhook processor transitioning orders to `Paid` upon verified `payment_intent.succeeded` events.
- [ ] **Failed Payment Recovery:** Automated customer notification and payment retry window upon `payment_intent.payment_failed`.
- [ ] **Automated Refunds & Partial Cancellations:** Backend service and admin action to trigger gateway refunds with inventory rollback.
- [ ] **Payment Reconciliation Audit:** Scheduled job reconciling internal order settlement records against payment processor balance transactions.

### 3. Cloud Storage, Email & Third-Party Adapters
- [x] **Transactional Email Engine:** Production Resend adapter (`infrastructure::email::ResendAdapter`) with non-blocking HTTP dispatch.
- [ ] **Email Provider Secondary Failover:** Secondary SMTP / AWS SES fallback provider ensuring 99.99% transactional email deliverability.
- [ ] **Cloud Object Storage (AWS S3 / Cloudflare R2):** Concrete `StoragePort` implementation replacing mock `S3Adapter` for product images and videos.
- [ ] **Pre-Signed Upload URLs:** Secure direct-from-browser S3 upload endpoints eliminating backend bandwidth bottlenecks for media files.
- [ ] **Automated Media Processing Pipeline:** Automated resizing, WebP/AVIF transcoding, and Cloudflare CDN caching for uploaded product assets.

### 4. Observability, Telemetry & Incident Runbooks
- [x] **Lockless Prometheus Metrics Registry:** Lockless in-memory metrics registry (`/metrics`) strictly bounded to <63 timeseries to prevent cardinality explosions.
- [x] **SIEM Audit Event Taxonomy:** 23 frozen security audit events (`auth.<subsystem>.<action>`) logging critical operational actions.
- [x] **Automatic Secret Redaction Layer:** Tracing subscriber masking 17 sensitive keywords (`password`, `token`, `secret`, `pepper`, etc.) across JSON and standard log outputs.
- [x] **Prometheus Production Alert Rules:** Pre-configured alerts (`ops/alerts/auth.yml`) covering P0 (Token Replay, 5xx rate), P1 (Argon2 queue saturation), and P2 (brute force).
- [x] **Grafana Dashboards:** Production dashboards for operational health (`auth-health.json`) and security monitoring (`auth-security.json`).
- [x] **Documented Operational Runbooks:** Verified incident response playbooks in `docs/runbooks/`:
  - [x] `signing-key-rotation.md`: Zero-downtime JWT signing key rollover.
  - [x] `mass-logout-incident.md`: Emergency global session invalidation.
  - [x] `suspected-account-breach.md`: Account compromise triage and forensic containment.
  - [x] `redis-outage.md`: Operating under degraded Redis cache availability.
  - [x] `postgres-failover.md`: Database failover recovery protocol.
- [ ] **Distributed OpenTelemetry (OTel) Tracing:** Trace-ID propagation across frontend network calls, Axum route middleware, and database SQLx queries.
- [ ] **Centralized Exception Tracking:** Sentry integration for real-time frontend and backend uncaught panic reporting.
- [ ] **Database Connection Pool & Query Alarms:** Automated alerting on database pool exhaustion and slow queries (>250ms).

### 5. Production Infrastructure, CI/CD & Deployment
- [ ] **Production Multi-Stage Dockerfile (Backend):** Highly optimized distroless/scratch container for Axum binary (<50MB image footprint).
- [ ] **Production Frontend NGINX / Caddy Container:** Multi-stage build producing static Vite bundle served via hardened web server with Brotli/Gzip compression.
- [ ] **Continuous Integration (CI) Workflow:** GitHub Actions pipeline executing `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace`, and frontend TypeScript typechecks.
- [ ] **Continuous Deployment (CD) Pipeline:** Automated container image building, vulnerability scanning (Trivy), and staging deployment.
- [ ] **Automated Database Backups & PITR:** Scheduled PostgreSQL snapshots with Write-Ahead Logging (WAL) archiving for Point-In-Time Recovery.
- [ ] **Kubernetes / ECS Orchestration Manifests:** Declarative deployment manifests with CPU/memory resource limits and health probes (`/health`, `/metrics`).
- [ ] **Enterprise Secrets Management:** Integration with AWS Secrets Manager, HashiCorp Vault, or Doppler for runtime secret injection.

---

## 🔵 Priority 2: Future Implementation

Next-generation capabilities, luxury customer experience innovations, international expansion, and enterprise platform extensions.

### 1. Immersive Luxury Experience & Merchandising
- [ ] **Interactive 3D WebGL Product Viewer:** High-fidelity 3D model rendering (Three.js / React Three Fiber) with 360-degree rotation, texture zooming, and lighting customization.
- [ ] **Augmented Reality (AR) "Try-At-Home" Preview:** Native AR Quick Look (iOS USDZ) and Scene Viewer (Android GLB) for mobile model visualization.
- [ ] **Bespoke Made-to-Measure Customizer:** Interactive size calculator capturing custom shoulder-to-floor lengths, bust, and sleeve measurements for haute couture abayas.
- [ ] **Custom Embroidery & Monogramming Request Flow:** Checkout add-on allowing customers to specify custom embroidery text and placement preview.
- [ ] **Customer Wishlist & Saved Collections:** User-authenticated wishlist sync with back-in-stock notification triggers.
- [ ] **Verified Buyer Reviews & Community Gallery:** Verified review submissions with star ratings, fit evaluations, and customer photo uploads.
- [ ] **VIP Tier & Loyalty Points Engine:** Points accumulation on purchases, tiered benefits (Bronze, Silver, Gold, Platinum), and member-only early access.

### 2. Global Commerce & Localization
- [ ] **Multi-Currency Pricing Engine:** Real-time FX exchange rate conversion with local currency price formatting (AED, SAR, QAR, USD, EUR, GBP).
- [ ] **Multilingual Storefront (i18n):** Full internationalization supporting Arabic (RTL layout) and English (LTR layout) with dynamic language toggling.
- [ ] **International Duty & Tax Engine:** Integration with tax calculation APIs (Avalara / Stripe Tax) for automated GCC VAT and international import customs duties.
- [ ] **Country-Specific Payment Methods:** Local payment integrations for GCC regions (Mada, Benefit, KNET, Apple Pay, Tabby/Tamara Buy-Now-Pay-Later).

### 3. Supply Chain, Logistics & Inventory Operations
- [ ] **Multi-Warehouse Inventory Allocation:** Regional inventory partitioning with intelligent order routing to the closest fulfillment facility.
- [ ] **Automated Reorder Threshold Alerts:** Dynamic inventory reorder point calculations notifying procurement teams when stock runs low.
- [ ] **Direct 3PL Carrier Integration:** Automated shipping label generation, dispatch manifests, and live tracking number syncing with DHL Express, FedEx, and Aramex.
- [ ] **Customer Self-Service RMA Returns:** Customer portal for initiating returns, printing return shipping labels, and tracking refund status.

### 4. Search, Intelligence & Personalization
- [ ] **Vector Semantic Product Search:** Fast, typo-tolerant semantic search powered by `pgvector` or Meilisearch with Arabic and English NLP tokenizers.
- [ ] **AI-Powered Recommendation Engine:** "Complete the Look" and personalized product recommendation carousels based on browsing history and affinity graphs.
- [ ] **AI Conversational Concierge:** Intelligent styling assistant and customer service chatbot integrated into the storefront.

### 5. Enterprise Integrations & Governance
- [ ] **Enterprise Single Sign-On (SSO):** SAML 2.0 and OIDC integrations (Google Workspace, Okta, Azure AD) for staff and administrative logins.
- [ ] **External Outgoing Webhook Subscription System:** Outgoing webhook management for third-party ERP (SAP, NetSuite) and CRM systems.
- [ ] **Automated Data Privacy & GDPR/CCPA Workflows:** Self-service "Export My Data" package generation and automated cryptographic data erasure workflows.
- [ ] **Comprehensive Financial Accounting Export:** Monthly balance sheet, tax summary, and payout reconciliation reports in CSV/Excel formats.

---

## 🛠️ Verification & Compliance Audit Trail

To verify the state of any feature in this checklist:
1. **Backend Integration Tests:**
   ```bash
   cargo test --workspace --tests
   ```
2. **Backend Static Verification:**
   ```bash
   cargo check --workspace
   cargo clippy --workspace -- -D warnings
   ```
3. **Frontend Type Check & Bundling:**
   ```bash
   cd frontend
   npm run typecheck
   npm run build
   ```
4. **Metrics & Health Verification:**
   ```bash
   curl -s http://localhost:3000/health
   curl -s http://localhost:3000/metrics | grep auth_
   ```

---

*This document is maintained as part of the HAS.LTD engineering specifications. Any structural changes to feature priority or operational status require review and sign-off by the lead system architect.*
