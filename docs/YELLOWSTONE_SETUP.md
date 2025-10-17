# Yellowstone gRPC Ingestion Setup

## Overview

This Python script connects to your local Yellowstone gRPC server (port 10000) and streams Pump.fun transactions to Postgres. This avoids all Rust dependency conflicts.

## Setup Instructions

### 1. Install Python Dependencies

```bash
pip install -r requirements.txt
```

### 2. Generate Yellowstone gRPC Proto Files

First, clone the Yellowstone gRPC repo:

```bash
cd /home/sol/Desktop/solana-dev
git clone https://github.com/rpcpool/yellowstone-grpc.git
```

Generate Python proto files:

```bash
cd /home/sol/Desktop/solana-dev/Bots/copytrader-bot
mkdir -p proto

python -m grpc_tools.protoc \
  -I../../../yellowstone-grpc/yellowstone-grpc-proto/proto \
  --python_out=./proto \
  --grpc_python_out=./proto \
  ../../../yellowstone-grpc/yellowstone-grpc-proto/proto/geyser.proto
```

### 3. Run the Ingester

```bash
python scripts/yellowstone_ingester.py
```

## How It Works

1. **Connects to Yellowstone gRPC** at `localhost:10000`
2. **Subscribes to transactions** involving Pump.fun program
3. **Filters for tracked wallets** (loaded from Postgres)
4. **Batches events** and writes to `raw_events` table every 5 seconds or 100 events
5. **No dependency conflicts** because it's pure Python!

## Advantages Over Rust

- ✅ No zeroize/sqlx/Solana dependency conflicts
- ✅ Much faster to iterate and debug
- ✅ Direct access to your existing Yellowstone gRPC setup
- ✅ Lower resource usage than enabling RPC PubSub
- ✅ Easy to modify and extend

## Next Steps

After ingestion is working:

1. Implement Pump.fun instruction decoder (can be in Python or Rust)
2. Build analytics layer to learn wallet patterns
3. Implement execution bot in Rust for speed
