use axum::{ routing::get, Router };

#[tokio::main]
async fn main() {
    let app = Router::new().route(
        "/health",
        get(|| async { "ok" })
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
