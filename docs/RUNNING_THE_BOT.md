# Discovery System - Quick Reference

## ✅ System Status

- **Bot**: Running and discovering wallets in real-time
- **Wallets Tracked**: 3,585+ (and growing)
- **Discovery Logging**: ✅ Enabled - shows "🆕 NEW WALLET DISCOVERED" messages

## 🚀 Running the Bot

### Option 1: Foreground (see all logs)

```bash
cd /home/sol/Desktop/solana-dev/Bots/copytrader-bot
./target/release/grpc_subscriber
```

### Option 2: Filtered Output (see only important events)

```bash
./target/release/grpc_subscriber 2>&1 | grep -E "(NEW WALLET|TRACKED WALLET|Flushed|error|ERROR)"
```

### Option 3: Background with Log File

```bash
./target/release/grpc_subscriber > bot.log 2>&1 &

# View logs
tail -f bot.log

# View only new discoveries
tail -f bot.log | grep "NEW WALLET"
```

### Option 4: Background with systemd (Recommended for Production)

Create `/etc/systemd/system/pumpfun-discovery.service`:

```ini
[Unit]
Description=Pump.fun Discovery Bot
After=network.target postgresql.service

[Service]
Type=simple
User=sol
WorkingDirectory=/home/sol/Desktop/solana-dev/Bots/copytrader-bot
ExecStart=/home/sol/Desktop/solana-dev/Bots/copytrader-bot/target/release/grpc_subscriber
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Then:

```bash
sudo systemctl daemon-reload
sudo systemctl enable pumpfun-discovery
sudo systemctl start pumpfun-discovery
sudo journalctl -u pumpfun-discovery -f  # View logs
```

## 🔍 Checking Discovery Progress

### Quick Stats

```bash
sudo -u postgres psql discovery -c "SELECT COUNT(*) FROM wallet_stats;"
```

### Top 10 Most Profitable Wallets

```bash
sudo -u postgres psql discovery -c "
SELECT
    wallet,
    total_trades,
    ROUND(net_pnl_sol::numeric, 4) as pnl_sol,
    ROUND(win_rate::numeric, 2) as win_rate,
    ROUND(profit_score::numeric, 2) as score
FROM wallet_stats
ORDER BY profit_score DESC
LIMIT 10;"
```

### Recently Active Wallets

```bash
sudo -u postgres psql discovery -c "
SELECT
    wallet,
    total_trades,
    buy_count,
    sell_count,
    ROUND(net_pnl_sol::numeric, 4) as pnl
FROM wallet_stats
ORDER BY last_seen DESC
LIMIT 10;"
```

### Wallets Ready for Promotion (Copy Trading Candidates)

```bash
sudo -u postgres psql discovery -c "
SELECT
    wallet,
    total_trades,
    ROUND(net_pnl_sol::numeric, 4) as pnl_sol,
    ROUND(win_rate::numeric, 2) as win_rate,
    ROUND(profit_score::numeric, 2) as score
FROM wallet_stats
WHERE total_trades >= 20
  AND win_rate >= 0.65
  AND net_pnl_sol >= 5.0
  AND NOT is_tracked
ORDER BY profit_score DESC
LIMIT 20;"
```

## 🛑 Stopping the Bot

### If running in foreground:

```bash
Ctrl+C
```

### If running in background:

```bash
pkill -9 grpc_subscriber
```

### If using systemd:

```bash
sudo systemctl stop pumpfun-discovery
```

## 📊 Log Messages Explained

- `🆕 NEW WALLET DISCOVERED` - A new wallet just made their first Pump.fun trade
- `🔔 TRACKED WALLET DETECTED` - One of your 308 tracked wallets made a trade
- `💾 Flushed X events to database` - Batch insert to copytrader DB completed
- `📊 Processed X transactions` - Progress counter (every 100 transactions)

## 🔧 Troubleshooting

### Bot won't start - password auth failed

Make sure postgres has proper permissions:

```bash
sudo -u postgres psql -c "GRANT ALL ON DATABASE discovery TO ahmad;"
```

### No new wallets being discovered

Check if discovery_url is set in config:

```bash
grep discovery_url configs/config.example.toml
```

### Database connection issues

Test connection:

```bash
sudo -u postgres psql discovery -c "SELECT 1;"
```

## 📈 Performance Monitoring

### Check database sizes

```bash
sudo -u postgres psql -c "
SELECT
    datname,
    pg_size_pretty(pg_database_size(datname)) as size
FROM pg_database
WHERE datname IN ('copytrader', 'discovery');"
```

### Active connections

```bash
sudo -u postgres psql -c "
SELECT datname, count(*)
FROM pg_stat_activity
WHERE datname IN ('copytrader', 'discovery')
GROUP BY datname;"
```

## 🎯 Next Steps

1. **Let it run for 24-48 hours** to gather meaningful statistics
2. **Analyze top performers** using the queries above
3. **Add profitable wallets** to your tracked list
4. **Build execution bot** to copy their trades

---

**Current Status**: ✅ Running and discovering ~50-100 new wallets per minute
**Database**: discovery (3,585+ wallets and growing)
**Last Updated**: October 17, 2025
