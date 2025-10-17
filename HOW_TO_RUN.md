# How to Run the Data Collection Bot

## Prerequisites

- Postgres database running with `copytrader` database created
- Yellowstone gRPC endpoint running on `localhost:10000`
- Config file at `configs/config.example.toml`

## Run Command

```bash
cd /home/sol/Desktop/solana-dev/Bots/copytrader-bot && RUST_LOG=info ./target/release/grpc_subscriber
```

## What It Does

- 🚀 Connects to Yellowstone gRPC and Postgres database
- 💰 Fetches live SOL price every 10 seconds from CoinGecko
- 👥 Monitors 308 tracked wallets for Pump.fun activity
- 🔔 Captures BUY/SELL/CREATE transactions in real-time
- 💾 Stores events in `raw_events` table with complete data:
  - Wallet aliases (names)
  - Pre/post SOL balances
  - Balance changes
  - Token amounts and prices
  - USD values
  - Transaction metadata

## Build First (if needed)

```bash
cargo build --release
```

## Stop the Bot

Press `Ctrl+C` to gracefully stop the bot.
