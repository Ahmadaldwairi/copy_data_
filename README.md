# Copytrader Bot Monorepo

Monorepo for ingestion (Rust), execution (Rust), and offline analytics (Python).

## Structure

- crates/: Rust workspace members
- sql/: SQLite and Postgres schemas
- configs/: runtime configs
- scripts/: DB init + migration helpers
- pytools/: analytics requirements
- data/: place external datasets like wallets.db

## Quick start

1. Build Rust workspace

```
cargo build
```

2. Prepare Postgres (optional now, recommended later)

- Create a database (e.g., `copytrader`).
- Initialize schema:

```
python scripts/init_postgres.py --pg-url postgresql://user:pass@localhost:5432/copytrader
```

3. Seed wallets (SQLite -> Postgres)

- Copy your source `wallets.db` into `data/` or pass the full path.

```
python scripts/seed_wallets.py --sqlite data/wallets.db --pg-url postgresql://user:pass@localhost:5432/copytrader
```

4. Run stubs

```
cargo run -p grpc_subscriber
cargo run -p exec_bot
```

## Config

See `configs/config.example.toml`. Set DB URL to sqlite for local dev, or postgres for prod.
