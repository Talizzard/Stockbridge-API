use axum::{extract::{Query, State}, http::StatusCode};
use sqlx::{Pool, Postgres};
use serde::Deserialize;
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

    // 2. Calcula exatamente que horas esse token vai morrer (Agpra + 21600 segundos)
    let expires_at = Utc::now() + Duration::seconds(token_data.expires_in as i64);
    let id = Uuid::new_v4().to_string();

    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 3. Limpa credenciais antigas (garantindo que a tabela só tenha 1 linha ativa)
    sqlx::query!("DELETE FROM meli_credentials").execute(&mut *tx).await.unwrap();

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
