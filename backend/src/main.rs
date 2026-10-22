mod auth;
mod db;
mod models;
mod routes;

use axum::{middleware, routing::get, Router};
use sqlx::MySqlPool;

async fn health() -> &'static str {
    "Server Rust Axum functional!"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool: MySqlPool = db::connect().await?;
    println!("Conectare MySQL reusita!");

    let private = Router::new()
        .merge(routes::clienti::router())
        .merge(routes::produse::router())
        .merge(routes::producatori::router())
        .merge(routes::asocieri::router())
        .merge(auth::private_router())
        .route_layer(middleware::from_fn_with_state(
            pool.clone(), auth::require_login,
        ));

    let app = Router::new()
        .route("/", get(health))
        .merge(auth::public_router())
        .merge(private)
        .with_state(pool);


    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Server pornit la http://localhost:3000");
    axum::serve(listener, app).await?;
    Ok(())
}
