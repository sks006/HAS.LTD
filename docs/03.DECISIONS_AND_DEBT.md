# Architectural Decisions & Technical Debt Register

This document tracks formal Architecture Decision Records (ADRs) and the Technical Debt Register for the **HAS.LTD** platform.

---

## Part 1: Architecture Decision Records (ADR)

### ADR-001: VARCHAR(50) + CHECK Constraints vs PostgreSQL ENUMs for Identity Fields (`role`, `status`)

* **Status:** Accepted
* **Date:** 2026-10-14
* **Applies To:** `users.role`, `users.status`, `backend/domain/src/models/user.rs`, `backend/migrations/`

#### Context
In initial designs, `users.role` was represented as `VARCHAR(50)` while `users.status` was defined using a native PostgreSQL `ENUM` type (`user_status_enum`). This caused architectural and operational inconsistencies:
1. Native PostgreSQL `ENUM` types cannot easily be updated inside transactional migration scripts (`ALTER TYPE ... ADD VALUE` cannot run inside transaction blocks in several PostgreSQL environments/versions).
2. Removing or renaming enum variants in PostgreSQL requires dropping dependent objects and re-creating the enum type.
3. Native enums bind the database catalog metadata directly to driver oids, complicating decoupling and schema evolution.

#### Decision
Standardize both `users.role` and `users.status` on **`VARCHAR(50)` with PostgreSQL `CHECK` constraints**:
* `chk_users_role`: `CHECK (role IN ('CUSTOMER', 'MODERATOR', 'ADMIN'))`
* `chk_users_status`: `CHECK (status IN ('ACTIVE', 'SUSPENDED', 'DELETED', 'PENDING'))`

In the Rust domain model (`backend/domain/src/models/user.rs`), both `UserRole` and `UserStatus` bridge text $\leftrightarrow$ enum explicitly:
* Implement `sqlx::Type<sqlx::Postgres>` delegating `type_info` and `compatible` to `String`.
* Implement `sqlx::Decode<'r, sqlx::Postgres>` using explicit string conversion.
* Implement `sqlx::Encode<'q, sqlx::Postgres>` writing the uppercase canonical variant.
* Implement `TryFrom<&str>`, `TryFrom<String>`, `FromStr`, `Display`, `as_str()`, and `AsRef<str>`.

#### Consequences
* **Positive:** Schema changes (such as introducing new roles or lifecycle states) are standard DDL updates executed safely within transactions with zero downtime.
* **Positive:** Complete consistency across identity schema attributes.
* **Positive:** Strict invariant enforcement both at the database level (`CHECK` constraint) and domain level (Rust enum).

---

## Part 2: Technical Debt Register (TD)

| ID | Component | Summary | Severity | Mitigation & Remediation Plan |
|---|---|---|---|---|
| **TD-001** | `backend/migrations/20261014000100_auth_schema.sql` | Backfill uses `gen_random_uuid()` (UUIDv4) instead of UUIDv7 | Low | The one-time backfill of legacy password hashes into `credentials` uses PostgreSQL's native `gen_random_uuid()` function, producing UUIDv4 identifiers. All runtime-generated credentials produced by the Rust domain layer use `CredentialId::now_v7()`. Legacy rows will be re-keyed or normalized during an upcoming maintenance data migration. |
| **TD-002** | `backend/domain/src/repositories/user_port.rs` | Extended user repository methods use default stubs | Low | Methods `update_status`, `update_role`, and `bump_auth_version` have default stub implementations pending full adapter integration in Phase 2/3. |
