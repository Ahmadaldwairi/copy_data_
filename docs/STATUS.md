# Copytrader Bot - Current Status & Next Steps

## ✅ Completed

### 1. Project Structure

- Created Rust workspace with 5 crates:
  - `common`: shared types, config loader, logging
  - `db`: Postgres/SQLite abstraction with sqlx
  - `decoder`: Pump.fun instruction decoder (stub)
  - `grpc_subscriber`: Yellowstone gRPC listener (in progress)
  - `exec_bot`: execution bot (stub)

### 2. Database Migration

- **Migrated 307 wallets** from SQLite to Postgres
- **Migrated 179 wallet roles** from SQLite to Postgres
- Created Postgres schema with tables:
  - `wallets` (wallet, alias, is_tracked, notes)
  - `wallet_roles` (wallet, role)
  - `raw_events` (ready for trade ingestion)
  - `trades` (for stitched P&L)
  - `wallet_patterns` (for learned fingerprints)
  - `follow_edges` (for copy-trader network)

### 3. Configuration

- Config loader in `common/src/config.rs`
- Example config at `configs/config.example.toml`
- Postgres URL: `postgresql://ahmad:Jadoo31991@localhost:5432/copytrader`
- Pump.fun program ID: `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P`

### 4. Database Layer

- Created `db/src/raw_events.rs` with insert functions
- RawEvent struct matches schema from highLevelSummary.md
- Batch insert capability for performance

## ⚠️ Current Blocker

**Dependency Conflict**: Yellowstone gRPC client and sqlx have incompatible transitive dependencies on `zeroize` crate versions. This prevents compilation.

### The Issue

- `yellowstone-grpc` v1.11-2.0 depend on `solana-sdk` v1.16-2.0
- `solana-sdk` requires `zeroize` <1.4
- `sqlx` v0.7-0.8 (via `rsa` crate) requires `zeroize` >=1.5

## 🔧 Solutions (Pick One)

### Option 1: Split into Separate Binaries (Recommended for MVP)

Create two separate processes:

1. **Ingestion service** (minimal deps):
   - Use `yellowstone-grpc` + simple Postgres driver (like `postgres` crate, not sqlx)
   - OR use RPC polling with `solana-client` + basic SQL
   - Writes to `raw_events` table only
2. **Analytics + Execution** (current workspace):
   - Keep sqlx for complex queries
   - Read from `raw_events`, write patterns, execute trades

### Option 2: Wait for Ecosystem Alignment

- Monitor for:
  - `yellowstone-grpc` update to use newer Solana SDK
  - `sqlx` downgrade or fix transitive deps
  - Solana SDK update to allow newer `zeroize`

### Option 3: Use RPC Polling Instead of gRPC

- Poll `getSignaturesForAddress` for each tracked wallet
- Fetch transaction details with `getTransaction`
- Simpler but higher latency (1-2s vs real-time)
- Good enough for learning patterns, may miss front-running opportunities

## 📋 Next Steps (Option 1 - Separate Binaries)

### A. Create lightweight ingestion binary

```bash
# New crate outside this workspace
cargo new --bin ingest_service
cd ingest_service
```

Add dependencies:

```toml
[dependencies]
yellowstone-grpc-client = "1.11"
yellowstone-grpc-proto = "1.11"
solana-sdk = "1.17"
postgres = "0.19"  # simpler, no sqlx
tokio = { version = "1", features = ["full"] }
serde_json = "1"
tracing = "0.1"
```

### B. Wire Yellowstone subscription

1. Connect to gRPC endpoint
2. Subscribe with filters:
   - `account_include`: list of 307 tracked wallets
   - `account_required`: Pump.fun program ID
3. Stream transactions
4. Basic decode (extract wallet, signature, slot, timestamp)
5. Insert raw rows into Postgres using `postgres` crate

### C. Keep current workspace for:

- Analytics (Python or Rust with sqlx queries)
- Pattern learning
- Execution bot

## 📝 What's in `highLevelSummary.md`

The ingestion bot should capture:

- **ts_ns**: chain slot-time or local receive timestamp (nanoseconds)
- **slot**: Solana slot number
- **sig**: transaction signature
- **wallet**: tracked wallet address (signer)
- **program**: Pump.fun program ID
- **action**: 'CREATE' | 'BUY' | 'SELL' | 'SWAP' | 'ADD' | 'REMOVE'
- **mint**: token mint address
- **amount_in**: SOL or token quantity in
- **amount_out**: SOL or token quantity out
- **price_est**: best-effort price at execution
- **fee_sol**: transaction fee
- **ix_accounts_json**: raw account list for debugging
- **meta_json**: extra decoded info

## 🎯 Immediate Action Items

1. **Decision**: Choose Option 1, 2, or 3 above
2. **If Option 1**:
   - Create separate `ingest_service` binary
   - Use `postgres` crate (not sqlx) for writes
   - Wire Yellowstone gRPC subscription
   - Test end-to-end: gRPC → decode → Postgres
3. **If Option 3** (simpler MVP):
   - Use `solana-client` RPC methods
   - Poll tracked wallets every 1-2 seconds
   - Fetch+decode transactions
   - Write to Postgres with simple SQL

## 📂 Files Ready to Use

- `sql/postgres_init.sql` - full schema
- `sql/postgres_wallet_roles.sql` - wallet roles table
- `scripts/seed_wallets.py` - migration script (already run)
- `configs/config.example.toml` - runtime config
- Database connection string in `.env`

## 🚀 Quick Test Command (once deps resolved)

```bash
export DATABASE_URL=postgresql://ahmad:Jadoo31991@localhost:5432/copytrader
cargo run -p grpc_subscriber
```

---

**Current workspace compiles except for grpc_subscriber due to the dep conflict.**
