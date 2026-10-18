use axum::{extract::State, Json, http::StatusCode};
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use crate::models::stock::{CreateMovementReq, MovementRes};

pub async fn create_movement(
    State(pool): State<Pool<Postgres>>,
                             Json(payload): Json<CreateMovementReq>,
) -> Result<Json<MovementRes>, (StatusCode, String)> {
    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let multiplier = if payload.movement_type == "IN" { 1 } else { -1 };
    let qty_change = payload.quantity * multiplier;

    let stock_id = Uuid::new_v4().to_string();
    let stock_query = sqlx::query!(
        r#"
        INSERT INTO stocks ("id", "productId", "locationId", "quantity")
    VALUES ($1, $2, $3, $4)
    ON CONFLICT ("productId", "locationId")
    DO UPDATE SET quantity = stocks.quantity + $4
    RETURNING quantity
    "#,
    stock_id, payload.product_id, payload.location_id, qty_change
    ).fetch_one(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if stock_query.quantity < 0 {
        return Err((StatusCode::BAD_REQUEST, "Estoque insuficiente para essa saída.".into()));
    }

    let movement_id = Uuid::new_v4().to_string();
    sqlx::query!(
        r#"INSERT INTO movements ("id", "productId", "locationId", "type", "quantity") VALUES ($1, $2, $3, $4, $5)"#,
                 movement_id, payload.product_id, payload.location_id, payload.movement_type, payload.quantity
    ).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(MovementRes { id: movement_id, new_balance: stock_query.quantity }))
}
