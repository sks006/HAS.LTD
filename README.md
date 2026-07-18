# HAS.LTD

HAS.LTD is a starter full-stack project that combines a Rust backend with a React/Vite frontend. It is structured as a modular workspace for building a modern web application with clear separation between API, domain, and infrastructure layers.

## Overview

This repository includes:

- A Rust workspace backend with three crates:
  - api: HTTP handlers and Axum routes
  - domain: business logic and core abstractions
  - infrastructure: database and external service integrations
- A React/Vite frontend with a simple component-based structure for UI development

## Project Structure

```text
.
├── Cargo.toml
├── Makefile
├── frontend/
│   ├── package.json
│   ├── vite.config.ts
│   └── src/
└── backend/
    ├── api/
    ├── domain/
    └── infrastructure/
```

## Prerequisites

Before getting started, make sure you have:

- Rust and Cargo installed
- Node.js and npm installed

## Getting Started

### 1. Install frontend dependencies

```bash
cd frontend
npm install
```

### 2. Run the frontend

```bash
npm run dev
```

The frontend will be available at http://localhost:5173.

### 3. Run the backend

From the repository root:

```bash
cargo run -p api
```

The API server will run at http://localhost:3000 and exposes a health endpoint at http://localhost:3000/health.

## Useful Commands

```bash
make frontend
make backend
```

## Development Notes

- The frontend is intentionally isolated from the backend and communicates via HTTP APIs.
- The backend is organized into separate crates to keep domain logic and infrastructure concerns decoupled.
- This project is a starting point and can be extended with databases, authentication, and domain-specific features.
