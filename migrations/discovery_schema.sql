-- Discovery Database Schema
-- Tracks ALL wallets for profitability analysis

-- Create discovery database
-- Run: createdb discovery

-- Wallet statistics
CREATE TABLE wallet_stats (
    wallet TEXT PRIMARY KEY,
    first_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Trade counts
    total_trades INTEGER NOT NULL DEFAULT 0,
    buy_count INTEGER NOT NULL DEFAULT 0,
    sell_count INTEGER NOT NULL DEFAULT 0,
    create_count INTEGER NOT NULL DEFAULT 0,
    
    -- P&L tracking (cumulative)
    total_sol_in DOUBLE PRECISION NOT NULL DEFAULT 0,   -- Total SOL spent on buys
    total_sol_out DOUBLE PRECISION NOT NULL DEFAULT 0,  -- Total SOL received from sells
    net_pnl_sol DOUBLE PRECISION NOT NULL DEFAULT 0,    -- sol_out - sol_in
    
    -- Realized performance (closed positions only)
    realized_wins INTEGER NOT NULL DEFAULT 0,
    realized_losses INTEGER NOT NULL DEFAULT 0,
    win_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    
    -- Tracking status
    is_tracked BOOLEAN NOT NULL DEFAULT FALSE,
    added_to_tracking_at TIMESTAMPTZ,
    
    -- Profitability score (for ranking)
    profit_score DOUBLE PRECISION NOT NULL DEFAULT 0,
    
    CONSTRAINT valid_win_rate CHECK (win_rate >= 0 AND win_rate <= 1),
    CONSTRAINT valid_counts CHECK (total_trades >= 0 AND buy_count >= 0 AND sell_count >= 0)
);

-- Indexes for performance
CREATE INDEX idx_wallet_stats_profit_score ON wallet_stats(profit_score DESC NULLS LAST)
WHERE total_trades >= 10;  -- Only rank wallets with sufficient activity

CREATE INDEX idx_wallet_stats_pnl ON wallet_stats(net_pnl_sol DESC)
WHERE total_trades >= 10;

CREATE INDEX idx_wallet_stats_last_seen ON wallet_stats(last_seen DESC);

CREATE INDEX idx_wallet_stats_tracked ON wallet_stats(is_tracked)
WHERE is_tracked = TRUE;

-- Position tracking for P&L calculation
CREATE TABLE positions (
    wallet TEXT NOT NULL,
    mint TEXT NOT NULL,
    bought_at TIMESTAMPTZ NOT NULL,
    
    -- Buy details
    token_amount DOUBLE PRECISION NOT NULL DEFAULT 0,
    sol_spent DOUBLE PRECISION NOT NULL,
    avg_buy_price DOUBLE PRECISION NOT NULL DEFAULT 0,  -- sol_spent / token_amount
    
    -- Sell details (when closed)
    is_closed BOOLEAN NOT NULL DEFAULT FALSE,
    sold_at TIMESTAMPTZ,
    sol_received DOUBLE PRECISION,
    realized_pnl DOUBLE PRECISION,
    
    PRIMARY KEY (wallet, mint, bought_at)
);

CREATE INDEX idx_positions_open ON positions(wallet, mint)
WHERE NOT is_closed;

CREATE INDEX idx_positions_wallet ON positions(wallet, bought_at DESC);

-- Daily aggregated stats for trend analysis
CREATE TABLE wallet_daily_stats (
    wallet TEXT NOT NULL,
    date DATE NOT NULL,
    trades INTEGER NOT NULL DEFAULT 0,
    buys INTEGER NOT NULL DEFAULT 0,
    sells INTEGER NOT NULL DEFAULT 0,
    sol_in DOUBLE PRECISION NOT NULL DEFAULT 0,
    sol_out DOUBLE PRECISION NOT NULL DEFAULT 0,
    daily_pnl DOUBLE PRECISION NOT NULL DEFAULT 0,
    
    PRIMARY KEY (wallet, date)
);

CREATE INDEX idx_daily_stats_date ON wallet_daily_stats(date DESC);
CREATE INDEX idx_daily_stats_pnl ON wallet_daily_stats(daily_pnl DESC);

-- View for top performers
CREATE VIEW top_profitable_wallets AS
SELECT 
    wallet,
    total_trades,
    net_pnl_sol,
    win_rate,
    profit_score,
    CAST(ROUND(CAST((total_sol_out / NULLIF(total_sol_in, 0) - 1) * 100 AS NUMERIC), 2) AS DOUBLE PRECISION) as roi_percent,
    last_seen,
    is_tracked
FROM wallet_stats
WHERE total_trades >= 10  -- Minimum trade threshold
ORDER BY profit_score DESC
LIMIT 100;

-- View for emerging traders (high win rate, lower volume)
CREATE VIEW emerging_traders AS
SELECT 
    wallet,
    total_trades,
    net_pnl_sol,
    win_rate,
    last_seen,
    is_tracked
FROM wallet_stats
WHERE 
    total_trades >= 5 
    AND total_trades < 20
    AND win_rate >= 0.6
    AND net_pnl_sol > 1.0  -- At least 1 SOL profit
ORDER BY win_rate DESC, net_pnl_sol DESC
LIMIT 50;
