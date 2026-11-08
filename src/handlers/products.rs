use axum::{extract::State, Json, http::StatusCode};
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use crate::models::stock::{CreateProductReq, ProductListItem, ProductRes};

pub async fn create_product(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<CreateProductReq>,
) -> Result<Json<ProductRes>, (StatusCode, String)> {
    if payload.name.trim().is_empty() || payload.sku.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Nome e SKU são obrigatórios.".to_string()));
    }
    if payload.cost < 0.0 || payload.price < 0.0 {
        return Err((StatusCode::BAD_REQUEST, "Custo e preço não podem ser negativos.".to_string()));
    }

    let new_id = Uuid::new_v4().to_string();

    sqlx::query!(
        r#"INSERT INTO products (id, name, sku, cost, price) VALUES ($1, $2, $3, $4, $5)"#,
        new_id, payload.name, payload.sku, payload.cost, payload.price
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("unique constraint") {
            (StatusCode::BAD_REQUEST, "Já existe um produto com esse SKU.".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, msg)
        }
    })?;

    Ok(Json(ProductRes { id: new_id }))
}

// Lista o catálogo com o estoque total de cada produto (somando todas as estantes)
pub async fn list_products(
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<Vec<ProductListItem>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, ProductListItem>(
        r#"
        SELECT p.id, p.name, p.sku, p.cost, p.price,
               COALESCE(SUM(s.quantity), 0)::int AS total_quantity
        FROM products p
        LEFT JOIN stocks s ON s."productId" = p.id
        GROUP BY p.id, p.name, p.sku, p.cost, p.price
        ORDER BY p.name
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows))
}
