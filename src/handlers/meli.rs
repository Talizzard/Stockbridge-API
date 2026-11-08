use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use sqlx::{FromRow, Pool, Postgres};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct MeliCallbackQuery {
    pub code: String,
}

pub async fn meli_callback(
    State(pool): State<Pool<Postgres>>,
    Query(query): Query<MeliCallbackQuery>,
) -> Result<String, (StatusCode, String)> {
    // 1. Vai na API do Meli e troca o código pelo Token
    let token_data = crate::services::meli::get_access_token(&query.code)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // 2. Calcula exatamente que horas esse token vai morrer (Agora + 21600 segundos)
    let expires_at = Utc::now() + Duration::seconds(token_data.expires_in as i64);
    let id = Uuid::new_v4().to_string();

    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 3. Limpa credenciais antigas (garantindo que a tabela só tenha 1 linha ativa)
    sqlx::query!("DELETE FROM meli_credentials")
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 4. Salva o novo combo de tokens no banco
    sqlx::query!(
        r#"INSERT INTO meli_credentials (id, access_token, refresh_token, expires_at) VALUES ($1, $2, $3, $4)"#,
        id, token_data.access_token, token_data.refresh_token, expires_at
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok("Integração concluída! Feche esta janela e volte para o sistema.".to_string())
}

// Diz se já existe uma conta do Meli conectada (nunca devolve os tokens)
#[derive(Serialize)]
pub struct MeliStatusRes {
    pub connected: bool,
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<String>,
}

pub async fn meli_status(
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<MeliStatusRes>, (StatusCode, String)> {
    let expires_at = sqlx::query_scalar::<_, String>(
        r#"SELECT to_char(expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') FROM meli_credentials LIMIT 1"#
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(MeliStatusRes { connected: expires_at.is_some(), expires_at }))
}

#[derive(Deserialize)]
pub struct CreateMeliLinkReq {
    pub product_id: String,
    pub meli_item_id: String, // Ex: MLB123456789
}

#[derive(Serialize)]
pub struct MeliLinkRes {
    pub id: String,
}

pub async fn create_meli_link(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<CreateMeliLinkReq>,
) -> Result<Json<MeliLinkRes>, (StatusCode, String)> {
    let item_id = payload.meli_item_id.trim().to_uppercase();
    if item_id.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "meli_item_id não pode ser vazio.".to_string()));
    }

    let link_id = Uuid::new_v4().to_string();

    // Insere o vínculo no banco de dados
    sqlx::query!(
        r#"INSERT INTO meli_links (id, "productId", "meliItemId") VALUES ($1, $2, $3)"#,
        link_id, payload.product_id, item_id
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("unique constraint") {
            // Tentaram cadastrar o mesmo MLB duas vezes
            (StatusCode::BAD_REQUEST, "Esse anúncio do Mercado Livre já está vinculado a um produto.".to_string())
        } else if msg.contains("foreign key") {
            (StatusCode::BAD_REQUEST, "Produto não encontrado.".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Erro no banco: {}", msg))
        }
    })?;

    Ok(Json(MeliLinkRes { id: link_id }))
}

#[derive(Serialize, FromRow)]
pub struct MeliLinkItem {
    pub id: String,
    #[serde(rename = "productId")]
    pub product_id: String,
    #[serde(rename = "meliItemId")]
    pub meli_item_id: String,
    #[serde(rename = "productName")]
    pub product_name: String,
    pub sku: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
}

pub async fn list_meli_links(
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<Vec<MeliLinkItem>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, MeliLinkItem>(
        r#"
        SELECT l.id,
               l."productId" AS product_id,
               l."meliItemId" AS meli_item_id,
               p.name AS product_name,
               p.sku,
               to_char(l."createdAt" AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
        FROM meli_links l
        JOIN products p ON p.id = l."productId"
        ORDER BY l."createdAt" DESC
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows))
}

pub async fn delete_meli_link(
    State(pool): State<Pool<Postgres>>,
    Path(link_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM meli_links WHERE id = $1")
        .bind(&link_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Vínculo não encontrado.".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
