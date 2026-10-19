use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use sqlx::{Pool, Postgres};
use chrono::Utc;

#[derive(Serialize)]
pub struct MeliUpdateStockReq {
    pub available_quantity: i32,
}

#[derive(Deserialize, Debug)]
pub struct MeliTokenRes {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: String,
    pub user_id: u64,
    pub refresh_token: String,
}

pub async fn get_access_token(auth_code: &str) -> Result<MeliTokenRes, String> {
    let client = Client::new();
    let app_id = env::var("MELI_APP_ID").expect("MELI_APP_ID não configurado");
    let secret = env::var("MELI_SECRET_KEY").expect("MELI_SECRET_KEY não configurado");

    // Pegamos a URL do env e fazemos o URL Encode básico (trocando : e /)
    let redirect_uri = env::var("MELI_REDIRECT_URI").expect("MELI_REDIRECT_URI não configurado");
    let safe_uri = redirect_uri.replace(":", "%3A").replace("/", "%2F");

    // Montamos o formulário manualmente em uma única String
    let body_payload = format!(
        "grant_type=authorization_code&client_id={}&client_secret={}&code={}&redirect_uri={}",
        app_id, secret, auth_code, safe_uri
    );

    let res = client
    .post("https://api.mercadolibre.com/oauth/token")
    .header("Content-Type", "application/x-www-form-urlencoded")
    .header("Accept", "application/json")
    .body(body_payload) // Enviamos o texto puro aqui!
    .send()
    .await
    .map_err(|e| format!("Erro de rede: {}", e))?;

    if res.status().is_success() {
        let token_data = res.json::<MeliTokenRes>().await.map_err(|e| format!("Erro no JSON: {}", e))?;
        Ok(token_data)
    } else {
        let error_text = res.text().await.unwrap_or_default();
        Err(format!("Erro da API Meli: {}", error_text))
    }
}

// Esta função será chamada toda vez que formos atualizar um anúncio
pub async fn get_valid_token(pool: &Pool<Postgres>) -> Result<String, String> {
    // Busca a credencial no banco (só deve existir uma linha)
    let cred = sqlx::query!(
        r#"SELECT access_token, refresh_token, expires_at FROM meli_credentials LIMIT 1"#
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Erro no banco de dados: {}", e))?;

    let cred = match cred {
        Some(c) => c,
        None => return Err("Nenhuma credencial encontrada. Faça o login no Meli primeiro!".into()),
    };

    // Verifica se ainda é válido (Damos uma margem de segurança de 5 minutos)
    if cred.expires_at > Utc::now() + chrono::Duration::minutes(5) {
        // Ainda está no prazo! Retorna o token instantaneamente sem gastar internet.
        return Ok(cred.access_token);
    }

    println!("Token Meli expirado ou quase vencendo. Iniciando renovação invisível...");

    // Se expirou, monta a requisição de renovação (refresh_token)
    let client = Client::new();
    let app_id = env::var("MELI_APP_ID").expect("MELI_APP_ID não configurado");
    let secret = env::var("MELI_SECRET_KEY").expect("MELI_SECRET_KEY não configurado");

    let body_payload = format!(
        "grant_type=refresh_token&client_id={}&client_secret={}&refresh_token={}",
        app_id, secret, cred.refresh_token
    );

    let res = client
    .post("https://api.mercadolibre.com/oauth/token")
    .header("Content-Type", "application/x-www-form-urlencoded")
    .header("Accept", "application/json")
    .body(body_payload)
    .send()
    .await
    .map_err(|e| format!("Erro de rede no refresh: {}", e))?;

    if res.status().is_success() {
        let token_data = res.json::<MeliTokenRes>().await.map_err(|e| format!("Erro JSON: {}", e))?;

        // Calcula a nova data de validade (Agora + Segundos retornados)
        let new_expires = Utc::now() + chrono::Duration::seconds(token_data.expires_in as i64);

        // Atualiza os dados oficiais no banco
        sqlx::query!(
            r#"UPDATE meli_credentials SET access_token = $1, refresh_token = $2, expires_at = $3"#,
            token_data.access_token, token_data.refresh_token, new_expires
        )
        .execute(pool)
        .await
        .map_err(|e| format!("Erro ao atualizar token no DB: {}", e))?;

        println!("Token renovado com sucesso!");
        Ok(token_data.access_token)
    } else {
        let err_text = res.text().await.unwrap_or_default();
        Err(format!("Falha na renovação Meli: {}", err_text))
    }
}

pub async fn update_item_stock(
    pool: &Pool<Postgres>,
    meli_item_id: &str,
    new_quantity: i32,
) -> Result<(), String> {
    // Pega o token mágico (ele se renova sozinho aqui dentro se precisar!)
    let token = get_valid_token(pool).await?;

    let client = Client::new();
    // A URL oficial do Meli para alterar um anúncio específico
    let url = format!("https://api.mercadolibre.com/items/{}", meli_item_id);

    let payload = MeliUpdateStockReq {
        available_quantity: new_quantity,
    };

    // Dispara o PUT enviando o Token no formato Bearer e o payload em JSON
    let res = client
    .put(&url)
    .header("Authorization", format!("Bearer {}", token))
    .header("Accept", "application/json")
    .json(&payload)
    .send()
    .await
    .map_err(|e| format!("Erro de rede ao conectar no Meli: {}", e))?;

    // Verifica se o Mercado Livre aceitou a mudança
    if res.status().is_success() {
        println!("Sucesso! Estoque do anúncio {} atualizado para {}.", meli_item_id, new_quantity);
        Ok(())
    } else {
        // Se der erro (ex: anúncio pausado ou ID errado), pegamos o motivo exato
        let err_text = res.text().await.unwrap_or_default();
        Err(format!("Meli rejeitou atualização do {}: {}", meli_item_id, err_text))
    }
}
