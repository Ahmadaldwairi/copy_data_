# Quick Start Guide - WebSocket Ingestion Bot

## ✅ Status: **READY TO RUN**

The WebSocket-based ingestion bot has been successfully compiled!

## What It Does

1. **Connects** to your Agave node's WebSocket endpoint (`ws://localhost:8900`)
2. **Subscribes** to transaction logs for the Pump.fun program
3. **Filters** transactions to only process ones from your 307 tracked wallets
4. **Fetches** full transaction details via RPC
5. **Writes** events to Postgres `raw_events` table

## Configuration

Edit `configs/config.example.toml`:

```toml
[solana]
rpc_url = "http://localhost:8899"
ws_url = "ws://localhost:8900"
chain = "mainnet"

[pumpfun]
program_id = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"

[database]
url = "postgresql://ahmad:Jadoo31991@localhost:5432/copytrader"
```

## Run It

```bash
# Set DATABASE_URL for compile-time checks
export DATABASE_URL=postgresql://ahmad:Jadoo31991@localhost:5432/copytrader

# Run the ingestion bot
cargo run -p grpc_subscriber
```

## What You'll See

```
🚀 Pump.fun ingestion bot starting up...
📝 Loaded config: ws_url=ws://localhost:8900, db_url=***
✅ Database connection healthy
👀 Loaded 307 tracked wallets
🎯 Monitoring Pump.fun program: 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P
🔌 Connecting to WebSocket: ws://localhost:8900
✅ WebSocket connected
📡 Subscribed to Pump.fun program logs, listening for transactions...
✅ Processed transaction: <signature>
💾 Flushed 15 events to database
```

## Next Steps

###1 Improve Transaction Parsing

Currently, the bot creates basic events with `action = "UNKNOWN"`. You need to:

- Decode Pump.fun instruction data to identify BUY/SELL/CREATE
- Extract mint addresses from transaction accounts
- Parse pre/post token balances to get amounts
- Calculate prices from bonding curve state

### 2. Add Pump.fun IDL

Get the Pump.fun program IDL and add proper instruction decoding in `crates/decoder/src/lib.rs`.

### 3. Test with Live Data

Make sure your Agave node is:

- Running
- Synced
- WebSocket endpoint accessible at `ws://localhost:8900`

### 4. Monitor Database

```sql
-- Check incoming events
SELECT COUNT(*), action FROM raw_events
GROUP BY action
ORDER BY COUNT(*) DESC;

-- Recent transactions
SELECT * FROM raw_events
ORDER BY ts_ns DESC
LIMIT 10;

-- Wallets with most activity
SELECT wallet, COUNT(*)
FROM raw_events
GROUP BY wallet
ORDER BY COUNT(*) DESC
LIMIT 10;
```

## Architecture

```
Agave Node (ws://localhost:8900)
         ↓
    [WebSocket Subscribe]
         ↓
  [Log Stream: Pump.fun txs]
         ↓
    [Filter: 307 wallets]
         ↓
  [Fetch full tx via RPC]
         ↓
  [Decode (TODO: improve)]
         ↓
   [Batch Buffer (100)]
         ↓
[Postgres: raw_events table]
```

## Troubleshooting

**"Connection refused" on WebSocket:**

- Check Agave is running: `solana cluster-info`
- Verify WebSocket port in config
- Try `ws://127.0.0.1:8900` if localhost doesn't work

**"Database connection failed":**

- Verify Postgres is running
- Check credentials in config
- Test: `psql postgresql://ahmad:Jadoo31991@localhost:5432/copytrader -c "SELECT 1"`

**"No transactions appearing":**

- Ensure Agave is synced
- Check Pump.fun program ID is correct
- Verify tracked wallets are active

## Files Modified

- `crates/grpc_subscriber/Cargo.toml` - Switched to solana-client (WebSocket)
- `crates/grpc_subscriber/src/main.rs` - WebSocket subscription logic
- `crates/db/Cargo.toml` - Downgraded sqlx to 0.6 for compatibility
- `configs/config.example.toml` - Added ws_url
- `crates/common/src/config.rs` - Updated config struct

---

**Ready to capture live Pump.fun trades!** 🚀
