use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, Header, EncodingKey};
use std::env;
use dotenvy::dotenv;

use domain::models::auth::JwtClaims;
use domain::models::user::UserRole;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let pool = PgPool::connect(&database_url).await?;
    println!("Connected to database.");

    // 1. Seed a test User
    let user_id = Uuid::new_v4();
    let email = format!("test-user-{}@has.ltd", Uuid::new_v4().to_string()[..8].to_string());
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, name, role, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, now(), now())
        "#
    )
    .bind(user_id)
    .bind(&email)
    .bind("$2b$12$L7R2b1pG8/1QvB1.k2qgOeW7qG8Jc.h2b.i1d1e1f1g1h1i1j1k1l") // dummy bcrypt/argon hash
    .bind("Test Customer")
    .bind("CUSTOMER") // customer role
    .bind(true)
    .execute(&pool)
    .await?;
    println!("Created user ID: {}", user_id);

    // 2. Seed a test Product
    let product_id = Uuid::new_v4();
    let slug = format!("test-product-{}", Uuid::new_v4().to_string()[..8].to_string());
    sqlx::query(
        r#"
        INSERT INTO products (id, slug, name, description, images, is_active, aggregate_rating, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, now(), now())
        "#
    )
    .bind(product_id)
    .bind(&slug)
    .bind("Test Premium Tee")
    .bind("A premium test t-shirt.")
    .bind(&vec!["http://example.com/image.jpg".to_string()])
    .bind(true)
    .bind(4.8)
    .execute(&pool)
    .await?;
    println!("Created product ID: {}", product_id);

    // 3. Seed a test Product Variant
    let variant_id = Uuid::new_v4();
    let sku = format!("TSHIRT-TEST-{}", Uuid::new_v4().to_string()[..8].to_string());
    sqlx::query(
        r#"
        INSERT INTO product_variants (id, product_id, sku, price_minor_units, currency, is_active, attributes, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, now(), now())
        "#
    )
    .bind(variant_id)
    .bind(product_id)
    .bind(&sku)
    .bind(2999) // $29.99
    .bind("USD")
    .bind(true)
    .bind(serde_json::json!({"size": "L", "color": "black"}))
    .execute(&pool)
    .await?;
    println!("Created variant ID: {} (SKU: {})", variant_id, sku);

    // 4. Seed an Inventory Stock Snapshot
    sqlx::query(
        r#"
        INSERT INTO inventory_stock_snapshots (variant_id, stock, reserved_stock, available_stock, version, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, now(), now())
        "#
    )
    .bind(variant_id)
    .bind(100) // stock
    .bind(0)   // reserved
    .bind(100) // available
    .bind(1)   // version
    .execute(&pool)
    .await?;
    println!("Created inventory snapshot for variant.");

    // 5. Generate a JWT signed access token
    let exp = (Utc::now() + Duration::days(7)).timestamp() as usize;
    let iat = Utc::now().timestamp() as usize;
    let claims = JwtClaims {
        sub: user_id,
        role: UserRole::Customer,
        jti: Uuid::new_v4(),
        iat,
        exp,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes())
    )?;

    println!("\n=== Generated Test JWT ===");
    println!("{}", token);
    println!("==========================\n");

    println!("Run the following checkout command:");
    println!(
        "curl -X POST http://127.0.0.1:3000/checkout \\\n  \
        -H \"Content-Type: application/json\" \\\n  \
        -H \"Authorization: Bearer {}\" \\\n  \
        -H \"Idempotency-Key: idempotency-key-{}\" \\\n  \
        -d '{{\n    \
          \"currency\": \"USD\",\n    \
          \"shipping_address\": {{\n      \
            \"recipient_name\": \"Shihab\",\n      \
            \"phone\": \"+1234567890\",\n      \
            \"street_line1\": \"123 Main St\",\n      \
            \"city\": \"Berlin\",\n      \
            \"postal_code\": \"10115\",\n      \
            \"country\": \"DE\"\n    \
          }},\n    \
          \"items\": [\n      \
            {{\n        \
              \"variant_id\": \"{}\",\n        \
              \"quantity\": 1\n      \
            }}\n    \
          ]\n        \
        }}'",
        token,
        Uuid::new_v4().to_string()[..8].to_string(),
        variant_id
    );

    Ok(())
}
