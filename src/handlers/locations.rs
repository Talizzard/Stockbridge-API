use axum::{extract::{State, Path, Query}, Json, http::StatusCode};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use crate::models::stock::{CreateLocationReq, LocationListItem, LocationRes, StockItemRes};

// Note o "pub" antes do async fn para que o main.rs consiga enxergar a função
pub async fn create_location(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<CreateLocationReq>,
) -> Result<Json<LocationRes>, (StatusCode, String)> {
    if payload.max_capacity <= 0 {
        return Err((StatusCode::BAD_REQUEST, "maxCapacity deve ser maior que zero.".to_string()));
    }

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
    .bind(&payload.max_capacity)
    .execute(&pool).await;

    match result {
        Ok(_) => Ok(Json(LocationRes { id: new_id })),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("unique constraint") {
                Err((StatusCode::BAD_REQUEST, "Já existe uma localização com esse fullCode ou qrToken.".to_string()))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, msg))
            }
        }
    }
}

// Filtro opcional: GET /api/locations?qrToken=XYZ (usado pelo app ao bipar a estante)
#[derive(Deserialize)]
pub struct LocationFilter {
    #[serde(rename = "qrToken")]
    pub qr_token: Option<String>,
}

pub async fn list_locations(
    State(pool): State<Pool<Postgres>>,
    Query(filter): Query<LocationFilter>,
) -> Result<Json<Vec<LocationListItem>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, LocationListItem>(
        r#"
        SELECT l.id,
               l."warehouseId" AS warehouse_id,
               l.sector, l.rack, l.level, l.position,
               l."fullCode" AS full_code,
               l."qrToken" AS qr_token,
               l."maxCapacity" AS max_capacity,
               l."isActive" AS is_active,
               COALESCE(SUM(s.quantity), 0)::int AS current_quantity
        FROM locations l
        LEFT JOIN stocks s ON s."locationId" = l.id
        WHERE ($1::text IS NULL OR l."qrToken" = $1)
        GROUP BY l.id, l."warehouseId", l.sector, l.rack, l.level, l.position,
                 l."fullCode", l."qrToken", l."maxCapacity", l."isActive"
        ORDER BY l."fullCode"
        "#
    )
    .bind(&filter.qr_token)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows))
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
