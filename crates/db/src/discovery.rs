//! Discovery database - track ALL wallets for profitability analysis

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// Wallet statistics for discovery
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WalletStats {
    pub wallet: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub total_trades: i32,
    pub buy_count: i32,
    pub sell_count: i32,
    pub create_count: i32,
    pub total_sol_in: f64,
    pub total_sol_out: f64,
    pub net_pnl_sol: f64,
    pub realized_wins: i32,
    pub realized_losses: i32,
    pub win_rate: f64,
    pub is_tracked: bool,
    pub profit_score: f64,
}

/// Update wallet stats after a trade
/// Returns true if this is a new wallet being discovered
pub async fn update_wallet_stats(
    pool: &PgPool,
    wallet: &str,
    action: &str,
    sol_amount: Option<f64>,
    mint: Option<&str>,
) -> Result<bool> {
    let sol_amt = sol_amount.unwrap_or(0.0);
    
    // Check if wallet exists before insert
    let exists: Option<i32> = sqlx::query_scalar("SELECT 1 FROM wallet_stats WHERE wallet = $1")
        .bind(wallet)
        .fetch_optional(pool)
        .await?;
    
    let is_new_wallet = exists.is_none();
    
    match action {
        "BUY" => {
            sqlx::query(
                r#"
                INSERT INTO wallet_stats (wallet, first_seen, last_seen, total_trades, buy_count, total_sol_in)
                VALUES ($1, NOW(), NOW(), 1, 1, $2)
                ON CONFLICT (wallet) DO UPDATE SET
                    last_seen = NOW(),
                    total_trades = wallet_stats.total_trades + 1,
                    buy_count = wallet_stats.buy_count + 1,
                    total_sol_in = wallet_stats.total_sol_in + $2
                "#
            )
            .bind(wallet)
            .bind(sol_amt)
            .execute(pool)
            .await?;
            
            // Track open position
            if let Some(m) = mint {
                sqlx::query(
                    r#"
                    INSERT INTO positions (wallet, mint, bought_at, token_amount, sol_spent, avg_buy_price)
                    VALUES ($1, $2, NOW(), 0, $3, 0)
                    ON CONFLICT (wallet, mint, bought_at) DO UPDATE SET
                        sol_spent = positions.sol_spent + $3
                    "#
                )
                .bind(wallet)
                .bind(m)
                .bind(sol_amt)
                .execute(pool)
                .await?;
            }
        }
        "SELL" => {
            sqlx::query(
                r#"
                INSERT INTO wallet_stats (wallet, first_seen, last_seen, total_trades, sell_count, total_sol_out)
                VALUES ($1, NOW(), NOW(), 1, 1, $2)
                ON CONFLICT (wallet) DO UPDATE SET
                    last_seen = NOW(),
                    total_trades = wallet_stats.total_trades + 1,
                    sell_count = wallet_stats.sell_count + 1,
                    total_sol_out = wallet_stats.total_sol_out + $2,
                    net_pnl_sol = wallet_stats.total_sol_out + $2 - wallet_stats.total_sol_in
                "#
            )
            .bind(wallet)
            .bind(sol_amt)
            .execute(pool)
            .await?;
            
            // Update position P&L
            if let Some(m) = mint {
                update_position_pnl(pool, wallet, m, sol_amt).await?;
            }
        }
        "CREATE" => {
            sqlx::query(
                r#"
                INSERT INTO wallet_stats (wallet, first_seen, last_seen, total_trades, create_count)
                VALUES ($1, NOW(), NOW(), 1, 1)
                ON CONFLICT (wallet) DO UPDATE SET
                    last_seen = NOW(),
                    total_trades = wallet_stats.total_trades + 1,
                    create_count = wallet_stats.create_count + 1
                "#
            )
            .bind(wallet)
            .execute(pool)
            .await?;
        }
        _ => {}
    }
    
    // Recalculate profit score
    recalculate_profit_score(pool, wallet).await?;
    
    Ok(is_new_wallet)
}

/// Update position P&L when selling
pub async fn update_position_pnl(
    pool: &PgPool,
    wallet: &str,
    mint: &str,
    sol_received: f64,
) -> Result<()> {
    // Find oldest open position for this mint (FIFO)
    let position = sqlx::query_as::<_, (DateTime<Utc>, f64)>(
        "SELECT bought_at, sol_spent FROM positions 
         WHERE wallet = $1 AND mint = $2 AND NOT is_closed 
         ORDER BY bought_at ASC LIMIT 1"
    )
    .bind(wallet)
    .bind(mint)
    .fetch_optional(pool)
    .await?;
    
    if let Some((bought_at, sol_spent)) = position {
        let pnl = sol_received - sol_spent;
        let is_win = pnl > 0.0;
        
        // Close position
        sqlx::query(
            r#"
            UPDATE positions SET
                is_closed = TRUE,
                sold_at = NOW(),
                sol_received = $3,
                realized_pnl = $4
            WHERE wallet = $1 AND mint = $2 AND bought_at = $5
            "#
        )
        .bind(wallet)
        .bind(mint)
        .bind(sol_received)
        .bind(pnl)
        .bind(bought_at)
        .execute(pool)
        .await?;
        
        // Update win/loss stats
        if is_win {
            sqlx::query(
                "UPDATE wallet_stats SET realized_wins = realized_wins + 1 WHERE wallet = $1"
            )
            .bind(wallet)
            .execute(pool)
            .await?;
        } else {
            sqlx::query(
                "UPDATE wallet_stats SET realized_losses = realized_losses + 1 WHERE wallet = $1"
            )
            .bind(wallet)
            .execute(pool)
            .await?;
        }
        
        // Update win rate
        sqlx::query(
            r#"
            UPDATE wallet_stats SET
                win_rate = CASE 
                    WHEN (realized_wins + realized_losses) > 0 
                    THEN realized_wins::float / (realized_wins + realized_losses)
                    ELSE 0
                END
            WHERE wallet = $1
            "#
        )
        .bind(wallet)
        .execute(pool)
        .await?;
    }
    
    Ok(())
}

/// Calculate profitability score for ranking
pub async fn recalculate_profit_score(pool: &PgPool, wallet: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE wallet_stats SET
            profit_score = CASE
                WHEN total_trades >= 10 THEN
                    (net_pnl_sol * win_rate * total_trades) / 100.0
                ELSE 0
            END
        WHERE wallet = $1
        "#
    )
    .bind(wallet)
    .execute(pool)
    .await?;
    
    Ok(())
}

/// Get top profitable wallets
pub async fn get_top_wallets(pool: &PgPool, limit: i32) -> Result<Vec<WalletStats>> {
    let wallets = sqlx::query_as::<_, WalletStats>(
        r#"
        SELECT wallet, first_seen, last_seen, total_trades, buy_count, sell_count, 
               create_count, total_sol_in, total_sol_out, net_pnl_sol,
               realized_wins, realized_losses, win_rate, is_tracked, profit_score
        FROM wallet_stats
        WHERE total_trades >= 10
        ORDER BY profit_score DESC
        LIMIT $1
        "#
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    
    Ok(wallets)
}
