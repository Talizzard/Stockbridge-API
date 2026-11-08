use axum::{extract::State, Json, http::StatusCode};
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use crate::models::stock::{CreateMovementReq, MovementRes, MovementHistoryRes};

pub async fn create_movement(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<CreateMovementReq>,
) -> Result<Json<MovementRes>, (StatusCode, String)> {
    // VALIDAÇÃO DO CORPO DA REQUISIÇÃO
    if payload.movement_type != "IN" && payload.movement_type != "OUT" {
        return Err((StatusCode::BAD_REQUEST, "O campo type deve ser \"IN\" ou \"OUT\".".to_string()));
    }
    if payload.quantity <= 0 {
        return Err((StatusCode::BAD_REQUEST, "A quantidade deve ser maior que zero.".to_string()));
    }

    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Trava a linha da estante (FOR UPDATE): duas entradas simultâneas na mesma estante
    // ficam em fila e não conseguem estourar a capacidade ao mesmo tempo.
    let loc = sqlx::query!(
        r#"SELECT "maxCapacity" FROM locations WHERE id = $1 FOR UPDATE"#,
        payload.location_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::BAD_REQUEST, "Localização não encontrada.".to_string()))?;

    //VALIDAÇÃO DE CAPACIDADE MÁXIMA
    if payload.movement_type == "IN" {
        // Soma as quantidades de TODOS os produtos que já estão guardados nela
        // O `COALESCE` e `::integer` garantem que se a estante estiver vazia, retorna 0 (e não null)
        let current_stock = sqlx::query!(
            r#"SELECT COALESCE(SUM(quantity), 0)::integer as "total!" FROM stocks WHERE "locationId" = $1"#,
            payload.location_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // Se a quantidade atual + o que estou tentando guardar estourar o limite, bloqueia
        if current_stock.total + payload.quantity > loc.maxCapacity {
            let error_msg = format!(
                "Capacidade excedida. A estante suporta {} itens, mas já tem {} guardados.",
                loc.maxCapacity, current_stock.total
            );
            return Err((StatusCode::BAD_REQUEST, error_msg));
        }
    }

    // Atualiza ou insere o saldo na tabela 'stocks' (Upsert)
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
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("foreign key") {
            (StatusCode::BAD_REQUEST, "Produto não encontrado.".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, msg)
        }
    })?;

    // Regra de negócio da Saída: não permitir estoque negativo
    // (ao retornar o erro, a transação é descartada e sofre rollback automático)
    if stock_query.quantity < 0 {
        return Err((StatusCode::BAD_REQUEST, "Estoque insuficiente para essa saída.".to_string()));
    }

    // Registra o histórico na tabela 'movements'
    let movement_id = Uuid::new_v4().to_string();
    sqlx::query!(
        r#"INSERT INTO movements ("id", "productId", "locationId", "type", "quantity") VALUES ($1, $2, $3, $4, $5)"#,
        movement_id, payload.product_id, payload.location_id, payload.movement_type, payload.quantity
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Efetiva a transação
    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // SINCRONIZAÇÃO COM O MERCADO LIVRE (em segundo plano)
    // Roda depois do commit e fora da requisição: se o Meli estiver lento ou fora do ar,
    // o operador do depósito recebe o "OK" do mesmo jeito. Falhas só aparecem no log.
    let pool_bg = pool.clone();
    let product_id_bg = payload.product_id.clone();
    tokio::spawn(async move {
        crate::services::meli::sync_product_stock(&pool_bg, &product_id_bg).await;
    });

    Ok(Json(MovementRes { id: movement_id, new_balance: stock_query.quantity }))
}

pub async fn get_movements(
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<Vec<MovementHistoryRes>>, (StatusCode, String)> {

    let history = sqlx::query_as!(
        MovementHistoryRes,
        r#"
        SELECT
        m.id,
        p.name as product_name,
        p.sku,
        m."type" as movement_type,
        m.quantity,
        p.cost,
        p.price,
        to_char(m."createdAt", 'YYYY-MM-DD"T"HH24:MI:SS"Z"') as created_at
        FROM movements m
        JOIN products p ON m."productId" = p.id
        ORDER BY m."createdAt" DESC
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(history))
}
