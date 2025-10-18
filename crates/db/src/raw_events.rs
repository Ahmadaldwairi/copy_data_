//! Database models and queries for raw_events

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct RawEvent {
    pub ts_ns: i64,
    pub slot: Option<i64>,
    pub sig: Option<String>,
    pub wallet: String,
    pub alias: Option<String>,         // Wallet alias/name for easy querying
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
    // New fields for complete event tracking
    pub block_time: Option<DateTime<Utc>>,  // Chain timestamp from metadata
    pub recv_time_ns: Option<i64>,    // Local receive timestamp for latency analysis
    pub ix_index: Option<i32>,        // Instruction index within transaction
    pub decode_ok: bool,               // Whether decode was successful
    pub decode_err: Option<String>,   // Error message if decode failed
    // Parsed balance fields for P&L accuracy (extracted from meta_json)
    pub pre_balance_sol: Option<f64>,  // Wallet SOL balance before transaction
    pub post_balance_sol: Option<f64>, // Wallet SOL balance after transaction
    pub balance_change_sol: Option<f64>, // Net SOL balance change (post - pre)
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
                ts_ns, slot, sig, wallet, alias, program, action,
                mint, base_mint, quote_mint, amount_in, amount_out,
                price_est, fee_sol, ix_accounts_json, meta_json, leader_wallet,
                block_time, recv_time_ns, ix_index, decode_ok, decode_err,
                pre_balance_sol, post_balance_sol, balance_change_sol
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25)
            ON CONFLICT (sig, wallet, action) DO NOTHING
            "#
        )
        .bind(event.ts_ns)
        .bind(event.slot)
        .bind(&event.sig)
        .bind(&event.wallet)
        .bind(&event.alias)
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
        .bind(event.block_time)
        .bind(event.recv_time_ns)
        .bind(event.ix_index)
        .bind(event.decode_ok)
        .bind(&event.decode_err)
        .bind(event.pre_balance_sol)
        .bind(event.post_balance_sol)
        .bind(event.balance_change_sol)
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
                ts_ns, slot, sig, wallet, alias, program, action,
                mint, base_mint, quote_mint, amount_in, amount_out,
                price_est, fee_sol, ix_accounts_json, meta_json, leader_wallet,
                block_time, recv_time_ns, ix_index, decode_ok, decode_err,
                pre_balance_sol, post_balance_sol, balance_change_sol
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25)
            ON CONFLICT (sig, wallet, action) DO NOTHING
            "#
        )
        .bind(event.ts_ns)
        .bind(event.slot)
        .bind(&event.sig)
        .bind(&event.wallet)
        .bind(&event.alias)
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
        .bind(event.block_time)
        .bind(event.recv_time_ns)
        .bind(event.ix_index)
        .bind(event.decode_ok)
        .bind(&event.decode_err)
        .bind(event.pre_balance_sol)
        .bind(event.post_balance_sol)
        .bind(event.balance_change_sol)
        .execute(pool)
        .await
        .map(|_| {
            count += 1;
        })
        .map_err(|e| {
            eprintln!("❌ Database insert error: {} (sig: {:?}, wallet: {}, action: {})", 
                e, event.sig, event.wallet, event.action);
            e
        })
        .ok();
    }
    Ok(count)
}
