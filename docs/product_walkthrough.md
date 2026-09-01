## Summary: Database Setup, Rust Backend, and Errors Encountered

### 1. Database Environment (Docker)
- **Technology**: PostgreSQL 16 running inside Docker (image: `postgres:16`), Redis for caching.
- **Docker Compose Setup**:
  - Services: `has_ltd_postgres` (container name `has_ltd`), `has_ltd_redis`.
  - Port mapping: Host `5432` → Container `5432` (PostgreSQL), Host `6379` → Container `6379` (Redis).
  - Volumes: Persistent storage for PostgreSQL (`postgres_data`) and Redis (`redis_data`).
  - Credentials: User `user`, password `test123`, database `has`.

- **Connection**: Accessible via `localhost:5432` from host; pgAdmin 4 (desktop) connects successfully.

### 2. Database Schema
- **Tables Created** (via migrations):
  - `users`: UUID `id`, `email` (unique), `password_hash`, `name`, `role` (string), `is_active`, `deleted_at`, `email_verified_at`, timestamps.
  - `products`: UUID `id`, `category_id` (UUID, FK to `categories`), `slug` (unique), `name`, `fabric_type`, `season`, `description`, `images` (text array), `featured_video_url`, `discount` (JSONB), `aggregate_rating`, `is_active`, timestamps, plus `price` (numeric) and `stock_quantity` (integer) added later.
  - `categories`: UUID `id`, `name` (unique), `description`, timestamps.

- **Sample Data**: Inserted 7 categories and 10 products across seasons with prices and stock quantities.

### 3. Rust Backend (Axum)
- **Framework**: Axum with SQLx for database access.
- **Routes** (from `build_router`):
  - `GET /health` – returns "ok"
  - `GET /products` – should list all products (handler: `list_products`)
  - `GET /products/{id}` – get single product by UUID (handler: `get_product`)
  - `POST /auth/register`, `POST /auth/login`, `POST /auth/logout`
  - `POST /cart/sync`, `POST /checkout`, `GET /orders/{id}`, `PUT /orders/{id}/status`
  - Admin and webhook routes.

- **State**: `AppState` holds a connection pool (`PgPool`).

### 4. What Works
- **Docker**: Containers run, database accessible.
- **Migrations**: Applied successfully via `sqlx migrate run`.
- **Data**: Categories and products inserted with correct relationships.
- **Backend health**: `GET /health` returns `ok`.
- **Authentication**: `POST /auth/login` returns a mock JWT token, indicating the handler works.

### 5. Issues Faced and Resolved
| Error | Cause | Solution |
|-------|-------|----------|
| `permission denied` connecting to Docker socket | User not in `docker` group | `sudo usermod -aG docker $USER` + `newgrp docker` |
| Port `5433` already in use | Native PostgreSQL running on host | Stopped native PostgreSQL (`sudo systemctl stop postgresql`) |
| `relation "categories" does not exist` | Categories table missing | Created `categories` table and inserted data |
| `duplicate key value violates unique constraint "products_slug_key"` | Slug conflicts in multiple attempts | Deleted old products or used new slugs |
| `column "price" does not exist` in queries | Missing `price` column | Added `price DECIMAL(10,2)` and `stock_quantity INTEGER` columns via `ALTER TABLE` |

### 6. Current Error – `/products` and `/products/{id}` Return Empty Reply
- **Symptom**: `curl http://localhost:3000/products` returns `curl: (52) Empty reply from server`. Same for product by ID.
- **Observation**: `/health` and `/auth/login` work fine; only product endpoints fail.
- **Likely Causes**:
  - Handler (`list_products` and `get_product`) uses a SQL query with columns that don't exist or have mismatched types (e.g., `price_minor_units` vs `price`, missing `currency`).
  - Serialization error: The Rust struct expects fields that aren't returned from the query or have incompatible types.
  - Database pool or connection issue specific to those queries (but `test-db` would confirm).
  - The server panics when processing the request, causing a connection reset (empty reply).

- **Next Steps to Debug**:
  - Check server logs in the `cargo run` terminal for panic messages.
  - Add a simple test endpoint (`/test-db`) to verify database connectivity.
  - Examine the actual `list_products` and `get_product` code to align with the current table schema.
  - Use `sqlx::query_as!` with a struct that matches the `products` table columns exactly.
  - Temporarily return mock data to isolate whether the issue is in the database query or the response formatting.

### 7. Overall Status
- The database is fully set up with sample data.
- The Rust backend is partially functional (auth, health).
- The product catalog endpoints are the next critical feature to fix.
- Once the product endpoints work, the remaining routes (checkout, cart, orders, admin) can be implemented and tested.

### 8. Key Learnings
- Always verify the database schema matches the query expectations.
- Use `cargo sqlx prepare` to keep compile-time checked queries in sync.
- For development, implement detailed error logging to quickly identify failures.
- Keep Docker and native services separate to avoid port conflicts.

