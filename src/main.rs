pub mod models;
pub mod handlers;
pub mod services;

use axum::{
    routing::{get, post},
    Router,
};
use axum::http::Method;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    // Tenta carregar o arquivo .env automaticamente (se a biblioteca dotenvy estiver instalada)
    // Se não estiver usando dotenvy, certifique-se de exportar as variáveis no terminal.
    let _ = dotenvy::dotenv();

    println!("Iniciando StockBridge API...");

    // 1. Configuração do Banco de Dados (PostgreSQL via Podman)
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL não configurada no .env");
    let pool = PgPoolOptions::new()
    .max_connections(5) // Mantém conexões abertas para alta performance
    .connect(&database_url)
    .await
    .expect("Falha ao conectar no banco de dados PostgreSQL");

    println!("✅ Banco de dados conectado!");

    // Roda as migrações pendentes automaticamente
    sqlx::migrate!("./migrations").run(&pool).await.expect("Falha ao rodar migrações do banco");

    println!("📦 Estrutura do banco de dados verificada/atualizada!");

    // 2. Configuração do CORS (Permite requisições do Frontend/React/Next.js)
    let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers(Any);

    // 3. Roteamento (Conectando os endpoints aos handlers)
    let app = Router::new()
    // Health Check
    .route("/api/health", get(|| async { "API Rust Operante!" }))

    // Produtos
    .route("/api/products", post(handlers::products::create_product))

    // Localizações e Estoque Físico
    .route("/api/locations", post(handlers::locations::create_location))
    .route("/api/locations/{id}/stock", get(handlers::locations::get_location_stock))

    // Movimentações (Entrada/Saída)
    .route("/api/movements", post(handlers::movements::create_movement))
    .route("/api/movements", get(handlers::movements::get_movements))

    // Autenticação e Integração Mercado Livre
    .route("/api/meli/callback", get(handlers::meli::meli_callback))
    .route("/api/meli/links", post(handlers::meli::create_meli_link))

    // Injeção de dependências e Middlewares
    .with_state(pool)
    .layer(cors); // CORS adicionado aqui como uma blindagem ao redor de tudo

    // 4. Inicialização do Servidor TCP
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Servidor rodando com alta performance na porta 3000!");
    axum::serve(listener, app).await.unwrap();
}
