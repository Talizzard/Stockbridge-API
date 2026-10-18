pub mod models;
pub mod handlers;
pub mod routes;

use axum::{routing::{get, post}, Router};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio::net::TcpListener;

// Importando as lógicas dos nossos novos arquivos
use crate::handlers::locations::{create_location, get_location_stock};
use crate::handlers::movements::create_movement;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL não configurada no .env");

    let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&db_url)
    .await
    .expect("Falha ao conectar no PostgreSQL");

    let app = Router::new()
    .route("/api/health", get(|| async { "API Rust Operante!" }))
    .route("/api/locations", post(create_location))
    .route("/api/locations/{id}/stock", get(get_location_stock))
    .route("/api/movements", post(create_movement))
    .with_state(pool);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Servidor Rust rodando na porta 3000!");
    axum::serve(listener, app).await.unwrap();
}
