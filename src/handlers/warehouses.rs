use axum::{extract::State, Json, http::StatusCode};
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use crate::models::stock::{CreateWarehouseReq, WarehouseItem, WarehouseRes};

pub async fn create_warehouse(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<CreateWarehouseReq>,
) -> Result<Json<WarehouseRes>, (StatusCode, String)> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "O nome do depósito não pode ser vazio.".to_string()));
    }

    let new_id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO warehouses (id, name) VALUES ($1, $2)")
        .bind(&new_id)
        .bind(&name)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(WarehouseRes { id: new_id }))
}

pub async fn list_warehouses(
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<Vec<WarehouseItem>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, WarehouseItem>("SELECT id, name FROM warehouses ORDER BY name")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows))
}
