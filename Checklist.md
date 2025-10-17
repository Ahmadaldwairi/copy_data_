#### Here is a checklist to read and implement:

#### First we need to make sure we are capturing the following {

    ✅ 1.Time -> slot ✅, block_time ✅ (column added), and recv_time_ns ✅ (IMPLEMENTED).

    ✅ 2.Actor -> The tracked wallet that initiated the action (signer you care about).(Optional) other signers if multi-sig or relays.

    ✅ 3.Mint/pair context -> Token mint being created/bought/sold.If swap/AMM: base_mint, quote_mint, pool address if known.

    ✅ 4.Action Type -> Enum: CREATE | BUY | SELL | SWAP | UNKOWN

    ✅ 5.Amount & Pricing -> amount_in ✅, amount_out ✅, fee_sol ✅. Exec price (price_est implemented).

    ✅ 6. Transaction Identity -> signature (unique), instruction index ✅, and a stable event id.

    ✅ 7. Pre/Post token balances (from meta) -> Stored in meta_json, can extract for PnL.

    ✅ 8. Logs & inner ixs -> Save meta_json/logs so you can re-decode edge cases later.

    ✅ 9. Accounts metas -> Raw ix_accounts (as JSON) is invaluable for future decoders.

    ✅ 10. Program Id -> Pump.fun program (and router/AMM program if involved).

    ✅ 11. Decode status -> decode_ok boolean ✅ + decode_err text ✅ (IMPLEMENTED).

    ✅ 12. Foreign keys -> Link to a canonical wallets table with alias (the public trader name you mentioned).

}

#### Postgres schema (drop-in) {

-- 1) Canonical wallets (seeded from your old SQLite)
CREATE TABLE IF NOT EXISTS wallets (
wallet TEXT PRIMARY KEY,
alias TEXT NOT NULL,
is_tracked BOOLEAN NOT NULL DEFAULT TRUE,
created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 2) Raw decoded events (append-only)
CREATE TABLE IF NOT EXISTS events_raw (
id BIGSERIAL PRIMARY KEY,
slot BIGINT,
block_time TIMESTAMPTZ, -- from chain if provided
recv_time_ns BIGINT, -- your local receive time
signature TEXT NOT NULL,
ix_index INT, -- which instruction in the tx
wallet TEXT NOT NULL REFERENCES wallets(wallet),
program TEXT NOT NULL, -- pump.fun or AMM program id
action TEXT NOT NULL, -- CREATE/BUY/SELL/SWAP/UNKNOWN
mint TEXT,
base_mint TEXT,
quote_mint TEXT,
amount_in NUMERIC(38,18),
amount_out NUMERIC(38,18),
fee_sol NUMERIC(38,9),
exec_price NUMERIC(38,18), -- derived
ix_accounts_json JSONB,
meta_json JSONB, -- logs, pre/post token balances, etc.
decode_ok BOOLEAN NOT NULL DEFAULT TRUE,
decode_err TEXT,
created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
UNIQUE(signature, ix_index, wallet, action)
);

-- Helpful indexes
CREATE INDEX IF NOT EXISTS idx_events_wallet_time ON events_raw(wallet, recv_time_ns);
CREATE INDEX IF NOT EXISTS idx_events_mint_time ON events_raw(mint, recv_time_ns);
CREATE INDEX IF NOT EXISTS idx_events_sig ON events_raw(signature);
CREATE INDEX IF NOT EXISTS idx_events_action_time ON events_raw(action, recv_time_ns);
CREATE INDEX IF NOT EXISTS idx_events_meta_gin ON events_raw USING GIN (meta_json);

-- 3) Trades (derived by your stitcher later)
CREATE TABLE IF NOT EXISTS trades (
trade_id BIGSERIAL PRIMARY KEY,
wallet TEXT NOT NULL REFERENCES wallets(wallet),
mint TEXT NOT NULL,
open_sig TEXT NOT NULL,
open_ts TIMESTAMPTZ NOT NULL,
open_qty NUMERIC(38,18) NOT NULL,
open_cost_sol NUMERIC(38,9) NOT NULL,
close_sig TEXT,
close_ts TIMESTAMPTZ,
close_qty NUMERIC(38,18),
close_proceeds_sol NUMERIC(38,9),
hold_ms BIGINT,
pnl_sol NUMERIC(38,9),
pnl_x NUMERIC(24,12),
was_win BOOLEAN,
meta_json JSONB,
UNIQUE(open_sig, wallet)
);
CREATE INDEX IF NOT EXISTS idx_trades_wallet_open ON trades(wallet, open_ts);

-- 4) Learned patterns (written by Analyzer)
CREATE TABLE IF NOT EXISTS wallet_patterns (
wallet TEXT PRIMARY KEY REFERENCES wallets(wallet),
sample_days INT,
sample_trades INT,
win_rate NUMERIC(6,4),
avg_daily_trades NUMERIC(12,4),
avg_entry_sol NUMERIC(24,9),
p50_hold_ms BIGINT, p90_hold_ms BIGINT,
p50_target_x NUMERIC(10,5), p90_target_x NUMERIC(10,5),
typical_exit_style TEXT,
slippage_pctl_95 NUMERIC(10,6),
followership_score NUMERIC(10,6),
leader_score NUMERIC(10,6),
preferred_time_window TEXT,
updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 5) Leader→Follower edges (written by Analyzer)
CREATE TABLE IF NOT EXISTS follow_edges (
leader TEXT NOT NULL REFERENCES wallets(wallet),
follower TEXT NOT NULL REFERENCES wallets(wallet),
lag_ms_p50 BIGINT,
lag_ms_p90 BIGINT,
follow_prob NUMERIC(6,4),
avg_follower_size_sol NUMERIC(24,9),
PRIMARY KEY (leader, follower)
);

}

#### 🧩 Overall Architecture {

--> Well-organized into /crates, /configs, /sql, /scripts, /docs.

--> Clear split between ingestion (Rust) and database schema — perfect for extending later with analyzer + exec bot.

--> Service and startup scripts (copytrader-bot.service, run_bot.sh) are correctly set up for daemonization.
}

#### ⚙️ Ingestor Focus (Main Review) {

✅ Captured:
1.Wallet, action type (BUY/SELL/UNKNOWN), signature, slot, timestamp, program ID.
2.Proper decoding pipeline and gRPC subscription through Yellowstone.
3.Writes to Postgres cleanly with async batching.
4.Includes decode_status, program name, and account metadata — good for debugging.

✅ ALL ITEMS COMPLETED:
✅ 1.Amount fields – amount_in ✅, amount_out ✅, and price_est ✅ (DONE)
✅ 2.Pre/Post token balances – stored in meta_json ✅, extracting pre/post SOL balances ✅ (DONE)
✅ 3.Fee field – fee_sol captured ✅ (DONE)
✅ 4.Dual timestamps – block_time ✅ and recv_time_ns ✅ columns added and implemented (DONE)
✅ 5.Link to aliases – wallet_aliases HashMap loaded and used ✅ (DONE)
✅ 6.Error tracking – decode_ok BOOLEAN ✅ and decode_err TEXT ✅ columns added and implemented (DONE)
}

#### 🗄️ SQL Layer {

✅ ALL ITEMS COMPLETED:
✅ -Index on (action, recv_time_ns) for fast time-based queries - ADDED
✅ -A decode_ok BOOLEAN column in events_raw - ADDED
✅ -A decode_err TEXT column in events_raw - ADDED
✅ -A foreign key constraint to wallets(wallet) - EXISTS
✅ -block_time TIMESTAMPTZ for chain timestamp - ADDED
✅ -recv_time_ns BIGINT for local receive timestamp - ADDED
✅ -ix_index INT for instruction tracking - ADDED
}

#### 🧠 Next Steps {

✅ 1.Add missing columns to the events schema - COMPLETED:
✅ - block_time TIMESTAMPTZ (chain time from meta) - ADDED
✅ - recv_time_ns BIGINT (local receive timestamp) - ADDED
✅ - decode_ok BOOLEAN - ADDED
✅ - decode_err TEXT - ADDED
✅ - ix_index INT (which instruction in the transaction) - ADDED

✅ 2.Extend the Rust RawEvent struct + Postgres insert to fill those fields - COMPLETED

✅ 3.Verify that for each Pump.fun transaction you have:
✅ -wallet
✅ -action
✅ -mint
✅ -slot
✅ -amount_in/out
✅ -block_time (column added)
✅ -recv_time_ns (implemented)
✅ -fee_sol
✅ -price_est
✅ -meta_json
✅ -decode_ok/decode_err
}#### ⚠️ Missing / Weak Spots {

| Aspect                            | Current                                                                   | Recommendation                                                                                                               |
| --------------------------------- | ------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| **Token amounts**                 | You parse instruction data but do **not compute amount_in / amount_out**. | Extract `token_amount` and/or SOL amount from instruction accounts or post-token-balances in `meta`. Store them numerically. |
| **Price derivation**              | Not implemented.                                                          | Derive `exec_price = amount_out / amount_in` (or from liquidity pool data if possible).                                      |
| **Pre/Post balances**             | Only raw `meta_json`.                                                     | Decode token balances into structured fields: pre_in, post_in, pre_out, post_out.                                            |
| **Fees**                          | Not tracked.                                                              | Add `fee_sol` from the meta logs (usually `fee: ... lamports`).                                                              |
| **Timestamp resolution**          | Only using local `SystemTime`.                                            | Add chain slot → block_time mapping from `meta.blockTime` (or RPC lookup once per N blocks).                                 |
| **Decode error tracking**         | You handle errors silently.                                               | Log + store in DB (`decode_ok=false`, `decode_err='...'`).                                                                   |
| **Unknown discriminator logging** | You silently return “unknown”.                                            | Log discriminator bytes to identify new Pump.fun instruction types automatically.                                            |

}

#### 🧠 3. Event Object → Database Mapping {

You’re using a struct like:
pub struct PumpEvent {
pub signature: String,
pub wallet: String,
pub action: String,
pub program: String,
pub mint: Option<String>,
pub slot: u64,
pub block_time: Option<i64>,
pub meta: Option<serde_json::Value>,
}

That’s good, but to match the full ingestion spec, extend it:
pub struct PumpEvent {
pub signature: String,
pub wallet: String,
pub alias: Option<String>,
pub action: String,
pub program: String,
pub mint: Option<String>,
pub base_mint: Option<String>,
pub quote_mint: Option<String>,
pub slot: u64,
pub block_time: Option<i64>,
pub recv_time_ns: i128,
pub amount_in: Option<f64>,
pub amount_out: Option<f64>,
pub fee_sol: Option<f64>,
pub exec_price: Option<f64>,
pub ix_accounts_json: Option<serde_json::Value>,
pub meta_json: Option<serde_json::Value>,
pub decode_ok: bool,
pub decode_err: Option<String>,
}

}

#### 🧩 4. Database Layer {

✅ You're using sqlx async pool — smart choice.

✅ Inserts use prepared statements — efficient.

✅ Schema has amount_in, amount_out, fee_sol, price_est — IMPLEMENTED

⚠️ Still need to add:

- block_time TIMESTAMPTZ (chain timestamp)
- recv_time_ns BIGINT (local receive time)
- decode_ok BOOLEAN
- decode_err TEXT
- ix_index INT
  }

#### ⚡ 5. gRPC Subscription {

You’re subscribing correctly to SubscribeTransactions.

Filtering logic:

It listens for Pump.fun program ID ✅

You filter only transactions where signers intersect tracked wallets ✅

You log slot + program ID + signature ✅

You parse and dispatch to the decoder ✅

Improvement suggestions:

Add backpressure protection (select! timeout or bounded channel).

Save latency: record both recv_time_ns and the block time (when available).
}

#### 🔄 6. Configuration and Performance {

You use environment configs via .env and dotenv ✅

You use WAL mode / Postgres connection pool ✅

Consider adding RUST_LOG=info level control (you can adjust verbosity live).
}

#### ✅ 7. Summary — Ingestor Audit {

| Requirement               | Status | Notes                                |
| ------------------------- | ------ | ------------------------------------ |
| Wallet captured           | ✅     | working with foreign key             |
| Action (BUY/SELL/UNKNOWN) | ✅     | works perfectly                      |
| Mint / program            | ✅     | captured correctly                   |
| Amounts                   | ✅     | amount_in, amount_out implemented    |
| Price                     | ✅     | price_est calculated                 |
| Fee                       | ✅     | fee_sol captured                     |
| Pre/Post balances         | ✅     | extracting SOL balances working      |
| Slot & chain time         | ✅     | slot ✅, block_time column added ✅  |
| Local receive time        | ✅     | recv_time_ns implemented ✅          |
| Decode status fields      | ✅     | decode_ok ✅ / decode_err ✅         |
| Account metas / meta logs | ✅     | stored in ix_accounts_json/meta_json |
| Aliases (names)           | ✅     | wallet_aliases loaded and displayed  |
| Instruction index         | ✅     | ix_index column added ✅             |

}

#### 🧩 8. Recommended Next Commit {

⚠️ **REMAINING PRIORITY ITEMS:**

1. Add decode_ok / decode_err tracking:

   - Add columns to raw_events table
   - Update RawEvent struct
   - Capture decode errors in grpc_subscriber

2. Add dual timestamps (block_time + recv_time_ns):

   - Add block_time TIMESTAMPTZ column (from transaction meta)
   - Add recv_time_ns BIGINT column (local receive time)
   - Separate from current ts_ns for latency analysis

3. Add ix_index INT:

   - Track which instruction in the transaction
   - Useful for multi-instruction transactions

4. Improve unknown discriminator logging:
   - Log discriminator bytes when action is UNKNOWN
   - Helps identify new Pump.fun instruction types

✅ **ALREADY COMPLETED:**

- ✅ Add numeric fields (amount_in, amount_out, fee_sol, price_est) - DONE
- ✅ Extend RawEvent struct with those fields - DONE
- ✅ Decode token balances from meta (pre/post SOL balances) - DONE
- ✅ Insert alias name via wallet_aliases HashMap - DONE

Once the remaining priority items are done, your ingestor will be 100% compliant with the complete design.
}
