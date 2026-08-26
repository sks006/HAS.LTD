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
├── Cargo.toml                  # Cargo workspace manifest
├── Makefile                    # Task runner for dev/build commands
├── LICENSE                     # License terms
├── README.md                   # Project documentation
├── backend/                    # 🚀 Axum Backend Crate Workspace
│   ├── .env                    # Backend environment config
│   ├── docker-compose.yml      # Local services (Postgres, Redis)
│   ├── api/                    # Ingress Layer (Axum web server)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs         # Application entrypoint
│   │       ├── routes.rs       # HTTP Router mapping paths to handlers
│   │       ├── state.rs        # AppState sharing connections & repositories
│   │       ├── dtos.rs         # Request/Response data contracts
│   │       ├── extractors.rs   # Custom middleware (JWT auth, idempotency checks)
│   │       ├── errors.rs       # HTTP status mapping for domain errors
│   │       ├── handlers/       # Controllers executing checkout/catalog routes
│   │       │   ├── catalog.rs
│   │       │   └── checkout.rs
│   │       └── bin/            # Executable scripts (e.g. seeding db/tokens)
│   │           └── seed_and_token.rs
│   ├── domain/                 # Domain Core Layer (No framework dependencies)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── errors.rs       # Core application/domain errors
│   │       ├── models/         # Business domain entities & aggregates
│   │       │   ├── auth.rs
│   │       │   ├── user.rs
│   │       │   ├── product.rs
│   │       │   ├── inventory.rs
│   │       │   └── order.rs
│   │       ├── repositories/   # Ports/Traits for caching & database operations
│   │       │   ├── user_port.rs
│   │       │   ├── product_port.rs
│   │       │   ├── inventory_port.rs
│   │       │   ├── order_port.rs
│   │       │   ├── checkout_port.rs
│   │       │   └── cache_port.rs
│   │       └── services/       # Domain-driven orchestrators
│   │           └── checkout.rs
│   ├── infrastructure/         # Infrastructure Layer (External integration adapters)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── database/       # Repository adapters backed by SQLx / PostgreSQL
│   │       │   ├── connection.rs
│   │       │   ├── user_repo.rs
│   │       │   ├── product_repo.rs
│   │       │   ├── inventory_repo.rs
│   │       │   ├── order_repo.rs
│   │       │   └── checkout_adapter.rs
│   │       ├── cache/          # In-memory Redis repository implementations
│   │       │   └── redis_adapter.rs
│   │       └── redis/          # Additional Redis adapters
│   │           └── cache_adapter.rs
│   └── migrations/             # SQL database migration files
└── frontend/                   # 💻 React + TypeScript Single Page Application (Vite)
    ├── package.json            # Node scripts & project dependencies
    ├── tsconfig.json           # TypeScript configuration
    ├── vite.config.ts          # Vite configuration
    ├── index.html              # HTML shell template
    └── src/
        ├── main.tsx            # Front-end mount & initialization
        ├── admin/              # Admin Panel components, pages & hooks
        ├── storefront/         # Customer storefront pages & components
        ├── components/         # Common / global components
        ├── hooks/              # Global React custom hooks
        ├── api/                # Global API clients
        └── shared/             # Shared state, utilities, and API layouts
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
