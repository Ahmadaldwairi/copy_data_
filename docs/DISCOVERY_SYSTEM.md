# Integrated Discovery System

## Overview

ONE bot that does TWO things:

1. **Track 308 curated wallets** (detailed data → `copytrader` DB)
2. **Discover profitable wallets** (aggregated stats → `discovery` DB)

## Architecture

```
┌─────────────────────────────────────────┐
│  Yellowstone gRPC Stream                │
│  (ALL Pump.fun transactions)            │
└──────────────┬──────────────────────────┘
               │
               ▼
      ┌────────────────┐
      │ grpc_subscriber │ (ONE process)
      │                 │
      │  1. Decode once │
      │  2. Check if    │
      │     tracked     │
      │  3. Store both  │
      └────┬───────┬────┘
           │       │
    ┌──────┘       └──────┐
    ▼                     ▼
┌─────────────┐    ┌──────────────┐
│ copytrader  │    │  discovery   │
│  Database   │    │   Database   │
│             │    │              │
│ 308 wallets │    │ ALL wallets  │
│ Full detail │    │ Stats only   │
└─────────────┘    └──────────────┘
```

## Benefits

✅ **Efficient**: One gRPC connection, one decoding pass
✅ **Complete**: No data loss - capture everything
✅ **Scalable**: Aggregate stats vs full events
✅ **Discoverable**: Find profitable wallets automatically
✅ **Maintainable**: One codebase, one deployment

## Implementation Steps

### 1. Setup Discovery Database

```bash
# Create database
createdb discovery

# Run migration
psql discovery < migrations/discovery_schema.sql
```

### 2. Update Config

```toml
# configs/config.example.toml
[database]
url = "postgres://localhost/copytrader"
discovery_url = "postgres://localhost/discovery"
```

### 3. Modify Bot Logic

In `grpc_subscriber/src/main.rs`:

```rust
// Connect to BOTH databases
let copytrader_pool = database::connect(Some(&config.database.url)).await?;
let discovery_pool = database::connect(Some(&config.database.discovery_url)).await?;

// In transaction processing:
for wallet in &found_wallets {
    for decoded in &decoded_actions {
        // Check if tracked
        let is_tracked = tracked_wallets.contains(wallet);

        if is_tracked {
            // Store detailed event
            create_raw_event(&copytrader_pool, ...).await?;
        }

        // ALWAYS update discovery stats (for ALL wallets)
        db::discovery::update_wallet_stats(
            &discovery_pool,
            wallet,
            decoded.action.as_str(),
            sol_amount,
            mint,
        ).await?;
    }
}
```

### 4. Query Top Performers

```sql
-- Get top 20 profitable wallets
SELECT * FROM top_profitable_wallets LIMIT 20;

-- Find emerging traders
SELECT * FROM emerging_traders;

-- Get wallet details
SELECT
    wallet,
    total_trades,
    net_pnl_sol,
    win_rate,
    profit_score,
    is_tracked
FROM wallet_stats
WHERE wallet = 'WALLET_ADDRESS';
```

### 5. Auto-Promote Wallets

Create a daily job to add top performers to tracking:

```sql
-- Find wallets to promote
SELECT wallet FROM wallet_stats
WHERE
    total_trades >= 20
    AND win_rate >= 0.65
    AND net_pnl_sol >= 5.0
    AND NOT is_tracked
ORDER BY profit_score DESC
LIMIT 10;

-- Then insert into copytrader.tracked_wallets
```

## Data Flow

### For Tracked Wallets (308):

```
Transaction → Decode → Store in copytrader.raw_events (full detail)
                    → Update discovery.wallet_stats (aggregated)
```

### For Non-Tracked Wallets (thousands):

```
Transaction → Decode → Update discovery.wallet_stats ONLY (aggregated)
```

## Database Sizes

**copytrader** (detailed):

- ~1000 events/hour from 308 wallets
- ~24k events/day
- ~8.7M events/year
- Estimated size: ~10-20 GB/year

**discovery** (aggregated):

- One row per wallet (updated in-place)
- ~1000 new wallets/day
- ~365k wallets/year
- Estimated size: ~100-500 MB/year

## Performance

- **No overhead**: Same gRPC stream, same decoding
- **Fast updates**: Simple UPDATE queries (indexed)
- **Efficient storage**: Stats vs full events (100x smaller)

## Future Enhancements

1. **Auto-promotion**: Automatically add top performers to tracked list
2. **Auto-demotion**: Remove underperforming wallets from tracking
3. **Alerts**: Notify when new profitable wallet discovered
4. **Dashboard**: Web UI showing top performers in real-time
5. **Backtesting**: Validate profitability calculations on historical data

## Next Steps

1. ✅ Create discovery database schema
2. ✅ Add discovery module to db crate
3. ⏳ Update main.rs to connect to both DBs
4. ⏳ Modify transaction processing to update both DBs
5. ⏳ Test with live data
6. ⏳ Build analytics queries
7. ⏳ Create promotion workflow
