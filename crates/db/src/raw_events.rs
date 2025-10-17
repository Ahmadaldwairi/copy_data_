//! Database models and queries for raw_events

use anyhow::Result;
use serde_json::Value as JsonValue;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct RawEvent {
    pub ts_ns: i64,
    pub slot: Option<i64>,
    pub sig: Option<String>,
    pub wallet: String,
    pub program: String,
    pub action: String,
    pub mint: Option<String>,
    pub base_mint: Option<String>,
    pub quote_mint: Option<String>,
    pub amount_in: Option<f64>,
    pub amount_out: Option<f64>,
    pub price_est: Option<f64>,
    pub fee_sol: Option<f64>,
    pub ix_accounts_json: Option<JsonValue>,
    pub meta_json: Option<JsonValue>,
    pub leader_wallet: Option<String>,
}

pub async fn insert_raw_events_batch(pool: &PgPool, events: &[RawEvent]) -> Result<()> {
    if events.is_empty() {
        return Ok(());
    }

    let mut tx = pool.begin().await?;
    for event in events {
        sqlx::query(
            r#"
            INSERT INTO raw_events (
                ts_ns, slot, sig, wallet, program, action,
                mint, base_mint, quote_mint, amount_in, amount_out,
                price_est, fee_sol, ix_accounts_json, meta_json, leader_wallet
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (sig, wallet, action) DO NOTHING
            "#
        )
        .bind(event.ts_ns)
        .bind(event.slot)
        .bind(&event.sig)
        .bind(&event.wallet)
        .bind(&event.program)
        .bind(&event.action)
        .bind(&event.mint)
        .bind(&event.base_mint)
        .bind(&event.quote_mint)
        .bind(event.amount_in)
        .bind(event.amount_out)
        .bind(event.price_est)
        .bind(&event.ix_accounts_json)
        .bind(&event.meta_json)
        .bind(&event.leader_wallet)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn batch_insert_raw_events(pool: &PgPool, events: &[RawEvent]) -> Result<usize> {
    let mut count = 0;
    for event in events {
        sqlx::query(
            r#"
            INSERT INTO raw_events (
                ts_ns, slot, sig, wallet, program, action,
                mint, base_mint, quote_mint, amount_in, amount_out,
                price_est, fee_sol, ix_accounts_json, meta_json, leader_wallet
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (sig, wallet, action) DO NOTHING
            "#
        )
        .bind(event.ts_ns)
        .bind(event.slot)
        .bind(&event.sig)
        .bind(&event.wallet)
        .bind(&event.program)
        .bind(&event.action)
        .bind(&event.mint)
        .bind(&event.base_mint)
        .bind(&event.quote_mint)
        .bind(event.amount_in)
        .bind(event.amount_out)
        .bind(event.price_est)
        .bind(event.fee_sol)
        .bind(&event.ix_accounts_json)
        .bind(&event.meta_json)
        .bind(&event.leader_wallet)
        .execute(pool)
        .await
        .map(|_| {
            count += 1;
        })
        .ok();
    }
    Ok(count)
}
