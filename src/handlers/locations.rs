use axum::{extract::{State, Path}, Json, http::StatusCode};
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use crate::models::stock::{CreateLocationReq, LocationRes, StockItemRes};

// Note o "pub" antes do async fn para que o main.rs consiga enxergar a função
pub async fn create_location(
    State(pool): State<Pool<Postgres>>,
                             Json(payload): Json<CreateLocationReq>,
) -> Result<Json<LocationRes>, (StatusCode, String)> {
    let new_id = Uuid::new_v4().to_string();

    let result = sqlx::query(
        r#"INSERT INTO "locations" ("id", "warehouseId", "sector", "rack", "level", "position", "fullCode", "qrToken", "isActive", "maxCapacity")
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#
    )
    .bind(&new_id)
    .bind(&payload.warehouse_id)
    .bind(&payload.sector)
    .bind(&payload.rack)
    .bind(&payload.level)
    .bind(&payload.position)
    .bind(&payload.full_code)
    .bind(&payload.qr_token)
    .bind(true)
    .bind(&payload.max_capacity) // <- NOVO BIND AQUI
    .execute(&pool).await;

    match result {
        Ok(_) => Ok(Json(LocationRes { id: new_id })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn get_location_stock(
    State(pool): State<Pool<Postgres>>,
                                Path(location_id): Path<String>,
) -> Result<Json<Vec<StockItemRes>>, (StatusCode, String)> {
    let stocks = sqlx::query_as!(
        StockItemRes,
        r#"
        SELECT p.id as product_id, p.name as product_name, p.sku, s.quantity
        FROM stocks s
        JOIN products p ON s."productId" = p.id
        WHERE s."locationId" = $1
        "#,
        location_id
    )
    .fetch_all(&pool).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(stocks))
}
