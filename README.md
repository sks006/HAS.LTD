# HAS.LTD Enterprise Scaffold

HAS.LTD is a production-grade full-stack boilerplate combining a **Rust** backend with a **React/Vite** frontend. It leverages clean architectural principles, domain-driven design (DDD), and transaction-safe database patterns (such as Optimistic Concurrency Control and Idempotent execution).

---

## 🏗️ Architectural Blueprint

The backend is structured around **Ports and Adapters (Hexagonal Architecture)** to keep business logic completely decoupled from external frameworks, protocols, and storage drivers.

```mermaid
graph TD
    Client[HTTP Client] -->|Axum HTTP Routes| Ingress[API Layer]
    Ingress -->|Commands / Queries| Domain[Domain Core]
    Domain -->|Ports / Traits| Repository[Infrastructure Layer]
    Repository -->|SQLx / Redis Client| DB[(PostgreSQL / Redis)]
```

### Layer Breakdown
*   **`domain`**: The core runtime and business logic. It contains entities, domain enums, validation constraints, and abstract traits (Ports) defining the interfaces for cache and databases. It has zero external third-party framework dependencies except for standard data types like `chrono` and `uuid`.
*   **`infrastructure`**: Implementation of the domain's Ports (Adapters). This layer contains the physical database drivers, SQLx adapters, Redis cache connection pools, and database repositories.
*   **`api`**: The Axum web server ingress. It maps incoming requests to DTOs, validates payload constraints, injects the state, evaluates extractors (Authentication, Idempotency), and invokes domain services.

---

## 📂 Directory Layout

```text
├── Cargo.toml                          # Workspace manifest coordinating backend crates
├── Makefile                            # Build automation, migration runners, and security audits
├── docker-compose.yml                  # Local network isolation for PostgreSQL, Redis, and LocalStack
├── README.md                           # System invariants and deployment specifications
│
├── backend/                            # 🚀 Axum Backend Workspace
│   ├── api/                            # INGRESS & TRANSPORT LAYER (Axum HTTP)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs                 # Server bootstrap, signal trapping, graceful shutdown
│   │       ├── routes.rs               # Route registration and middleware pipeline assembly
│   │       ├── state.rs                # Shared AppState (database pools, cache, crypto adapters)
│   │       ├── dtos/                   # Strict ingress validation boundaries
│   │       │   ├── auth_dto.rs         # Login/Registration payloads (Zxcvbn-checked)
│   │       │   ├── catalog_dto.rs      # Pagination, filtering, and sorting parameters
│   │       │   ├── cart_dto.rs         # Item mutations and quantity deltas
│   │       │   ├── checkout_dto.rs     # Shipping info, payment tokens, idempotency headers
│   │       │   └── admin_dto.rs        # Product CRUD and inventory reconciliation payloads
│   │       ├── middleware/             # DEFENSIVE SECURITY LAYER
│   │       │   ├── rate_limiter.rs     # Redis-backed distributed token bucket
│   │       │   ├── secure_headers.rs   # CSP, HSTS, X-Frame-Options, X-Content-Type-Options
│   │       │   ├── cors.rs             # Strict origin, method, and credential whitelisting
│   │       │   ├── auth_guard.rs       # JWT/PASETO extractor and claims validator
│   │       │   ├── rbac.rs             # Role-Based Access Control (Customer vs. Admin)
│   │       │   └── idempotency.rs      # Distributed lock preventing duplicate charges
│   │       ├── handlers/               # Thin HTTP controllers (Delegates to domain services)
│   │       │   ├── auth.rs             # Authentication lifecycle (Login, Refresh, Logout)
│   │       │   ├── catalog.rs          # Public product exploration
│   │       │   ├── cart.rs             # Cart state synchronization
│   │       │   ├── checkout.rs         # Atomic order placement and transaction kickoff
│   │       │   ├── admin.rs            # Protected administrative operations
│   │       │   └── webhooks.rs         # Payment gateway cryptographic signature verification
│   │       └── errors.rs               # RFC 7807 Problem Details HTTP error mapper
│   │
│   ├── domain/                         # BUSINESS DOMAIN LAYER (Zero Framework Dependencies)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── errors.rs               # Strongly typed domain-level business errors
│   │       ├── models/                 # Pure domain entities, value objects, and aggregates
│   │       │   ├── user.rs             # User aggregate, PasswordHash, Email value objects
│   │       │   ├── role.rs             # RBAC permissions and identity claims
│   │       │   ├── product.rs          # Product aggregate, Money value object, SKU
│   │       │   ├── inventory.rs        # Stock reserve entities and allocation locks
│   │       │   ├── cart.rs             # Ephemeral cart calculations and tax boundaries
│   │       │   ├── order.rs            # Order state machine (Pending, Paid, Shipped, Cancelled)
│   │       │   └── payment.rs          # Payment Intent, Settlement proofs, and Transaction IDs
│   │       ├── repositories/           # Ports/Abstract Trait boundaries
│   │       │   ├── user_port.rs        # User identity storage contract
│   │       │   ├── product_port.rs     # Catalog retrieval contract
│   │       │   ├── inventory_port.rs   # Row-level stock reservation contract
│   │       │   ├── order_port.rs       # Atomic transactional persistence contract
│   │       │   ├── cache_port.rs       # Ephemeral key-value cache contract
│   │       │   ├── storage_port.rs     # Direct S3 object streaming contract
│   │       │   ├── hasher_port.rs      # Password hashing interface (Argon2id)
│   │       │   └── payment_port.rs     # Payment gateway communication interface
│   │       └── services/               # Domain-driven business orchestrators
│   │           ├── auth_service.rs     # Session verification and credential checks
│   │           ├── checkout_service.rs # Distributed inventory reservation & payment orchestration
│   │           └── catalog_service.rs  # Business logic for dynamic pricing and discounts
│   │
│   ├── infrastructure/                 # ADAPTER & INTEGRATION LAYER
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── database/               # PostgreSQL adapters via SQLx (Connection pooling)
│   │       │   ├── connection.rs       # Pool initialization with retry policies & SSL mode
│   │       │   ├── user_repo.rs        # SQL implementation for user management
│   │       │   ├── product_repo.rs     # Read-optimized catalog queries
│   │       │   ├── inventory_repo.rs   # Pessimistic locking (SELECT FOR UPDATE) queries
│   │       │   └── order_repo.rs       # Multi-table atomic transactional operations
│   │       ├── cache/                  # Redis adapters
│   │       │   ├── redis_pool.rs       # Redis multiplexed connection manager
│   │       │   ├── session_adapter.rs  # Blacklist and refresh token storage
│   │       │   └── lock_adapter.rs     # Redlock/atomic SET NX distributed locks
│   │       ├── security/               # Cryptographic security adapters
│   │       │   ├── argon2_hasher.rs    # PHC string formatting and salt generation
│   │       │   └── token_provider.rs   # Ed25519 signing / asymmetric JWT verification
│   │       ├── storage/                # Cloud storage integration
│   │       │   └── s3_adapter.rs       # Zero-copy chunked stream pipe to S3/Cloudflare R2
│   │       └── payment/                # Payment processor integration
│   │           └── gateway_adapter.rs  # Secure HMAC-SHA256 signature verification
│   │
│   └── migrations/                     # SQLx migration scripts (Schema definitions, constraints, indexes)
│
frontend/
├── package.json
├── tsconfig.json
├── vite.config.ts                      # Strict proxy routing to Axum to bypass CORS in dev
├── index.html
└── src/
    ├── main.tsx                        # DOM mount, strict mode, and global error boundary catch
    │
    ├── app/                            # LAYER 1: APP INITIALIZATION
    │   ├── router.tsx                  # Global route definitions (React Router)
    │   └── providers/                  # Context wrappers (Theme, React Query, Auth)
    │       ├── AuthProvider.tsx
    │       └── QueryClientProvider.tsx
    │
    ├── shared/                         # LAYER 2: CROSS-DOMAIN INFRASTRUCTURE
    │   ├── api/                        # Axios/Fetch network interceptors
    │   │   ├── client.ts               # Injects JWT Bearer tokens into headers automatically
    │   │   └── endpoints.ts            # Hardcoded route strings
    │   ├── types/                      # Exact TypeScript mappings of your Rust DTOs
    │   │   ├── contracts.ts            # ProductDto, OrderDto, Uuid
    │   │   └── roles.ts                # 'SUPER_ADMIN' | 'MODERATOR' | 'CUSTOMER'
    │   ├── store/                      # Global Zustand primitive orchestrators
    │   │   └── store.ts                # Binds isolated slices together
    │   └── ui/                         # Dumb, stateless, highly reusable components
    │       ├── Button.tsx
    │       ├── Modal.tsx
    │       └── Table.tsx
    │
    ├── features/                       # LAYER 3: ISOLATED BUSINESS DOMAINS
    │   ├── auth/
    │   │   ├── api/                    # Login, Refresh, Logout mutations
    │   │   ├── store/                  # Zustand: authSlice (JWT, Role, UserID)
    │   │   ├── components/             # LoginForm, RegisterForm
    │   │   └── guards/                 # RBAC enforcement wrappers
    │   │       ├── RequireAdmin.tsx    # Blocks Moderator & Customer
    │   │       ├── RequireModerator.tsx# Blocks Customer (Allows Admin/Moderator)
    │   │       └── RequireAuth.tsx     # Blocks unauthenticated traffic
    │   │
    │   ├── catalog/                    # Public browsing (Customer & Guest)
    │   │   ├── api/                    # fetchProducts, fetchProductById
    │   │   ├── store/                  # Zustand: catalogSlice (Active filters, search query)
    │   │   └── components/             # ProductGrid, FilterSidebar, 3DViewerFallback
    │   │
    │   ├── cart_checkout/              # Ephemeral state & OCC processing
    │   │   ├── api/                    # syncCart, submitOrder (with idempotency_key)
    │   │   ├── store/                  # Zustand: cartSlice (Optimistic UI updates)
    │   │   └── components/             # CartDrawer, PaymentGatewayForm
    │   │
    │   ├── moderation/                 # Moderator access zone
    │   │   ├── api/                    # fetchAssignedOrders, toggleProductStatus
    │   │   └── components/             # OrderAuditQueue, ProductReviewTable
    │   │
    │   └── admin/                      # God-mode operational zone
    │       ├── api/                    # deleteRole, modifyInventory, systemMetrics
    │       └── components/             # AccessControlMatrix, GlobalRevenueDashboard
    │
    └── pages/                          # LAYER 4: ROUTE ASSEMBLIES
        ├── storefront/
        │   ├── Home.tsx                # Composes features/catalog components
        │   ├── ProductDetail.tsx
        │   └── Checkout.tsx
        ├── dashboard/
        │   ├── ModeratorPanel.tsx      # Wrapped in <RequireModerator>
        │   └── SystemGodMode.tsx       # Wrapped in <RequireAdmin>
        └── auth/
            └── Login.tsx
```

---

## ⚡ Core Technologies

### Backend
*   **Rust (Edition 2021)**: The core compiled language.
*   **Axum**: Highly concurrent web application framework built on top of `tokio`, `tower`, and `hyper`.
*   **SQLx**: SQL toolkit with async drivers and connection pooling for PostgreSQL. Queries are run dynamically at runtime to support rapid compilation.
*   **Redis**: In-memory data structures store used for token revocation and API idempotency lockups.
*   **Serde**: Serialization and deserialization framework used for processing JSON fields (e.g. metadata/address structures) stored in `JSONB` database fields.
*   **jsonwebtoken**: Cryptographic signature validation for JWTs.

### Frontend
*   **React + TypeScript**: Declarative UI components.
*   **Vite**: Frontend build tool and dev server.

---

## ⚙️ Features & Design Patterns

### 1. Identity, Auth & RBAC
*   **JWT Revocation Matrix**: User authentication tokens are verified against a Redis blacklist. If a token ID (`jti`) is found on the Redis blacklist, it is rejected immediately.
*   **Intent Isolation**: Verification tokens (e.g. Email verification, password reset) are stored in the database with strict `TokenPurpose` mapping to prevent token re-use across different security contexts.

### 2. High-Frequency Cache Matrix
*   **Idempotency Engine**: Idempotency keys submitted by clients are validated against a Redis cache with TTL enforcement. If a collision is detected, the adapter returns the original cached result, preventing double-billing or duplicate resource creation.

### 3. Product Catalog Aggregates
*   **Vectorized Data Access**: Bulk operations fetch products and variants efficiently using PostgreSQL `ANY($1)` vectorized queries to prevent `N+1` performance bottlenecks.
*   **Verified Review Validation**: To leave a product review, the repository verifies that the rating is within bounds (1–5), check duplicates, and ensures the reviewing user has a completed purchase history for the respective product.

### 4. Transaction-Safe Order Lifecycle
*   **Optimistic Concurrency Control (OCC)**: Order state transitions validate the version of the order model before modifying data. Any change increments the `version` field. If another worker writes to the order concurrently, the database update fails, rollback is triggered, and a conflict is thrown.
*   **Transactional Inventory Isolation**: Items added to orders lock the row for update, ensure state restrictions, and atomically modify the order totals within a database transaction.

---

## 🛠️ Installation & Getting Started

### Prerequisites
*   [Rust & Cargo](https://rustup.rs/) (v1.75+)
*   [Node.js](https://nodejs.org/) (v18+)
*   PostgreSQL & Redis (usually started via Docker Compose)

### 1. Configure the Environment
Create a `.env` file inside `backend/api/` (or set it in your system variables):
```env
DATABASE_URL=postgres://postgres:password@localhost:5432/has_ltd
JWT_SECRET=your_super_secret_jwt_signature_key_here
```

### 2. Set Up Frontend Dependencies
```bash
cd frontend
npm install
```

### 3. Build & Run Projects

#### Frontend Development Server
From the `frontend/` directory:
```bash
npm run dev
```
The client dashboard will be available at `http://localhost:5173`.

#### Backend API Server
From the repository root:
```bash
cargo run -p api
```
The Axum server boots up at `http://localhost:3000`.

---

## 💻 Useful Commands

Make use of the provided `Makefile` wrappers for convenience:

*   `make setup`: Run project scaffolding initialization tasks.
*   `make frontend`: Install dependencies and start the React dev server.
*   `make backend`: Start the Axum Rust API server.
*   `make dev`: Diagnostic developer command.
