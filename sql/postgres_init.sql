-- Postgres schema (primary store)
CREATE TABLE IF NOT EXISTS wallets (
  wallet TEXT PRIMARY KEY,
  alias TEXT,
  is_tracked BOOLEAN NOT NULL DEFAULT TRUE,
  notes TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_wallets_tracked ON wallets(is_tracked);

CREATE TABLE IF NOT EXISTS raw_events (
  id BIGSERIAL PRIMARY KEY,
  ts_ns BIGINT NOT NULL,
  slot BIGINT,
  sig TEXT,
  wallet TEXT NOT NULL REFERENCES wallets(wallet),
  program TEXT NOT NULL,
  action TEXT NOT NULL,
  mint TEXT,
  base_mint TEXT, quote_mint TEXT,
  amount_in DOUBLE PRECISION, amount_out DOUBLE PRECISION,
  price_est DOUBLE PRECISION,
  fee_sol DOUBLE PRECISION,
  ix_accounts_json JSONB,
  meta_json JSONB,
  leader_wallet TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(sig, wallet, action)
);
CREATE INDEX IF NOT EXISTS idx_raw_events_wallet_ts ON raw_events(wallet, ts_ns);
CREATE INDEX IF NOT EXISTS idx_raw_events_mint_ts ON raw_events(mint, ts_ns);

CREATE TABLE IF NOT EXISTS trades (
  trade_id BIGSERIAL PRIMARY KEY,
  wallet TEXT NOT NULL REFERENCES wallets(wallet),
  mint TEXT NOT NULL,
  open_sig TEXT NOT NULL,
  open_ts_ns BIGINT NOT NULL,
  open_qty DOUBLE PRECISION NOT NULL,
  open_cost_sol DOUBLE PRECISION NOT NULL,
  close_sig TEXT,
  close_ts_ns BIGINT,
  close_qty DOUBLE PRECISION,
  close_proceeds_sol DOUBLE PRECISION,
  hold_ms BIGINT,
  pnl_sol DOUBLE PRECISION,
  pnl_x DOUBLE PRECISION,
  was_win BOOLEAN,
  meta_json JSONB,
  UNIQUE(open_sig, wallet)
);
CREATE INDEX IF NOT EXISTS idx_trades_wallet_open ON trades(wallet, open_ts_ns);
CREATE INDEX IF NOT EXISTS idx_trades_mint_open ON trades(mint, open_ts_ns);

CREATE TABLE IF NOT EXISTS wallet_patterns (
  wallet TEXT PRIMARY KEY REFERENCES wallets(wallet),
  sample_days INTEGER,
  sample_trades INTEGER,
  win_rate DOUBLE PRECISION,
  avg_daily_trades DOUBLE PRECISION,
  avg_entry_sol DOUBLE PRECISION,
  p50_hold_ms BIGINT, p90_hold_ms BIGINT,
  p50_target_x DOUBLE PRECISION, p90_target_x DOUBLE PRECISION,
  typical_exit_style TEXT,
  slippage_pctl_95 DOUBLE PRECISION,
  followership_score DOUBLE PRECISION,
  leader_score DOUBLE PRECISION,
  preferred_time_window TEXT,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS follow_edges (
  leader TEXT NOT NULL REFERENCES wallets(wallet),
  follower TEXT NOT NULL REFERENCES wallets(wallet),
  lag_ms_p50 BIGINT,
  lag_ms_p90 BIGINT,
  follow_prob DOUBLE PRECISION,
  avg_follower_size_sol DOUBLE PRECISION,
  PRIMARY KEY (leader, follower)
);
