use axum::{extract::State, Json, http::StatusCode};
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use crate::models::stock::{CreateProductReq, ProductRes};

pub async fn create_product(
    State(pool): State<Pool<Postgres>>,
                            Json(payload): Json<CreateProductReq>,
) -> Result<Json<ProductRes>, (StatusCode, String)> {

    let new_id = Uuid::new_v4().to_string();

    sqlx::query!(
        r#"INSERT INTO products (id, name, sku, cost, price) VALUES ($1, $2, $3, $4, $5)"#,
                 new_id, payload.name, payload.sku, payload.cost, payload.price
    )
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ProductRes { id: new_id }))
}
