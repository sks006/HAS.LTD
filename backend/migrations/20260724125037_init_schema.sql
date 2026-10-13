-- 1. Custom Enumerations and Types
CREATE TYPE order_state_enum AS ENUM ('Pending', 'Reserved', 'Paid', 'Shipped', 'Cancelled');
CREATE TYPE ledger_action_enum AS ENUM ('Restock', 'Reserve', 'Release', 'Fulfill');

-- Enable implicit casts from TEXT/VARCHAR to order_state_enum so Rust can bind &str directly
CREATE OR REPLACE FUNCTION order_state_enum_cast(text) RETURNS order_state_enum AS $$
    SELECT $1::order_state_enum;
$$ LANGUAGE SQL IMMUTABLE;
CREATE CAST (text AS order_state_enum) WITH FUNCTION order_state_enum_cast(text) AS IMPLICIT;

-- 2. Users and Authentication
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    deleted_at TIMESTAMP WITH TIME ZONE,
    email_verified_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);

CREATE TABLE verification_tokens (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) UNIQUE NOT NULL,
    purpose VARCHAR(50) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);

-- 3. Product Catalog
CREATE TABLE products (
    id UUID PRIMARY KEY,
    category_id UUID,
    slug VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    fabric_type VARCHAR(100),
    season VARCHAR(100),
    description TEXT,
    images TEXT[] NOT NULL,
    featured_video_url TEXT,
    discount JSONB,
    aggregate_rating DOUBLE PRECISION DEFAULT 0.0 NOT NULL,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);

CREATE TABLE product_variants (
    id UUID PRIMARY KEY,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    sku VARCHAR(100) UNIQUE NOT NULL,
    price_minor_units INT NOT NULL,
    currency VARCHAR(10) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    attributes JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);

CREATE TABLE product_reviews (
    id UUID PRIMARY KEY,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    rating SMALLINT NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_verified_purchase BOOLEAN DEFAULT FALSE NOT NULL,
    comment TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    UNIQUE (product_id, user_id)
);

-- 4. Order and Checkout System
CREATE TABLE orders (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    state order_state_enum NOT NULL,
    currency VARCHAR(10) NOT NULL,
    shipping_address JSONB,
    billing_address JSONB,
    total_minor_units INT DEFAULT 0 NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    idempotency_key TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);

CREATE TABLE order_items (
    id UUID PRIMARY KEY,
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    variant_id UUID REFERENCES product_variants(id) ON DELETE SET NULL,
    quantity INT NOT NULL,
    product_name VARCHAR(255) NOT NULL,
    sku VARCHAR(100) NOT NULL,
    price_minor_units INT NOT NULL,
    currency VARCHAR(10) NOT NULL,
    thumbnail_url TEXT,
    attributes JSONB NOT NULL
);

-- 5. Inventory Ledger and OCC Snapshots
CREATE TABLE inventory_ledger (
    id UUID PRIMARY KEY,
    variant_id UUID NOT NULL REFERENCES product_variants(id) ON DELETE CASCADE,
    action ledger_action_enum NOT NULL,
    quantity_change INT NOT NULL,
    reference_id UUID,
    idempotency_key TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);

CREATE TABLE inventory_stock_snapshots (
    variant_id UUID PRIMARY KEY REFERENCES product_variants(id) ON DELETE CASCADE,
    stock INT NOT NULL,
    reserved_stock INT NOT NULL,
    available_stock INT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL
);
