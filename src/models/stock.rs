use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateLocationReq {
    #[serde(rename = "warehouseId")] pub warehouse_id: String,
    pub sector: String, pub rack: String, pub level: String,
    pub position: Option<String>,
    #[serde(rename = "fullCode")] pub full_code: String,
    #[serde(rename = "qrToken")] pub qr_token: String,
    #[serde(rename = "maxCapacity")] pub max_capacity: i32, // capacidade das estantes #valeu heverton <3
}

#[derive(Serialize)]
pub struct LocationRes { pub id: String }

#[derive(Deserialize)]
pub struct CreateMovementReq {
    #[serde(rename = "productId")] pub product_id: String,
    #[serde(rename = "locationId")] pub location_id: String,
    #[serde(rename = "type")] pub movement_type: String,
    pub quantity: i32,
}

#[derive(Serialize)]
pub struct MovementRes { pub id: String, pub new_balance: i32 }

#[derive(Serialize)]
pub struct StockItemRes {
    pub product_id: String,
    pub product_name: String,
    pub sku: String,
    pub quantity: i32,
}

#[derive(Serialize)]
pub struct MovementHistoryRes {
    pub id: String,
    #[serde(rename = "productName")]
    pub product_name: String,
    pub sku: String,
    #[serde(rename = "type")]
    pub movement_type: String,
    pub quantity: i32,
    pub cost: f64,
    pub price: f64,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateProductReq {
    pub name: String,
    pub sku: String,
    pub cost: f64,
    pub price: f64,
}

#[derive(Serialize)]
pub struct ProductRes {
    pub id: String,
}
