# Running the Copytrader Bot

## Quick Start Commands

### 1. Run Bot in Foreground (for testing)

```bash
cd /home/sol/Desktop/solana-dev/Bots/copytrader-bot
cargo run --release -p grpc_subscriber
```

### 2. Run Bot in Background with tmux (RECOMMENDED)

```bash
# Start new tmux session
tmux new -s copytrader

# Inside tmux, run the bot
cd /home/sol/Desktop/solana-dev/Bots/copytrader-bot
./run_bot.sh

# Detach from tmux: Press Ctrl+B, then D
# Reattach later: tmux attach -t copytrader
# Kill session: tmux kill-session -t copytrader
```

### 3. Run Bot in Background with nohup

```bash
cd /home/sol/Desktop/solana-dev/Bots/copytrader-bot
nohup ./run_bot.sh > logs/bot.log 2>&1 &

# Check if running
ps aux | grep grpc_subscriber

# View logs
tail -f logs/bot.log

# Stop the bot
pkill -f grpc_subscriber
```

### 4. Run as System Service (Production)

```bash
# Copy service file to systemd
sudo cp copytrader-bot.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Enable service (start on boot)
sudo systemctl enable copytrader-bot

# Start the service
sudo systemctl start copytrader-bot

# Check status
sudo systemctl status copytrader-bot

# View logs
sudo journalctl -u copytrader-bot -f

# Stop the service
sudo systemctl stop copytrader-bot

# Disable auto-start on boot
sudo systemctl disable copytrader-bot
```

## Monitoring

### View Live Logs (tmux method)

```bash
tmux attach -t copytrader
```

### View Live Logs (systemd method)

```bash
sudo journalctl -u copytrader-bot -f
```

### View Log Files (nohup method)

```bash
tail -f logs/bot.log
```

### Check Recent Trades

```sql
psql postgresql://ahmad:Jadoo31991@localhost:5432/copytrader -c "
SELECT
    action,
    LEFT(wallet, 8) as wallet,
    amount_in as tokens,
    ROUND(amount_out::numeric, 4) as sol,
    ROUND((amount_out * price_est)::numeric, 2) as usd,
    TO_CHAR(created_at, 'HH24:MI:SS') as time
FROM raw_events
WHERE amount_out IS NOT NULL
ORDER BY created_at DESC
LIMIT 10;
"
```

### Check Bot is Running

```bash
# Check process
ps aux | grep grpc_subscriber

# Check systemd service
sudo systemctl status copytrader-bot

# Check tmux session
tmux list-sessions
```

## Stopping the Bot

### Stop tmux session

```bash
tmux kill-session -t copytrader
```

### Stop nohup process

```bash
pkill -f grpc_subscriber
# or
kill $(ps aux | grep grpc_subscriber | grep -v grep | awk '{print $2}')
```

### Stop systemd service

```bash
sudo systemctl stop copytrader-bot
```

## Recommended: Use tmux

**Best for development and monitoring:**

```bash
# Start
tmux new -s copytrader
cd /home/sol/Desktop/solana-dev/Bots/copytrader-bot
./run_bot.sh

# Detach: Ctrl+B then D
# Reattach: tmux attach -t copytrader
```

**Advantages:**

- ✅ Easy to reattach and view live logs
- ✅ Survives SSH disconnects
- ✅ Auto-restarts on crash
- ✅ Full terminal output visible
- ✅ No root/sudo needed

## Troubleshooting

### Bot won't start

```bash
# Check if port 10000 is accessible (Yellowstone gRPC)
nc -zv localhost 10000

# Check database connection
psql postgresql://ahmad:Jadoo31991@localhost:5432/copytrader -c "SELECT 1;"

# Check config file exists
cat configs/config.example.toml
```

### View all running processes

```bash
ps aux | grep -E "(grpc_subscriber|copytrader)"
```

### Clean restart

```bash
# Stop all instances
pkill -f grpc_subscriber
sudo systemctl stop copytrader-bot 2>/dev/null || true
tmux kill-session -t copytrader 2>/dev/null || true

# Wait a moment
sleep 2

# Start fresh
tmux new -s copytrader
cd /home/sol/Desktop/solana-dev/Bots/copytrader-bot
./run_bot.sh
```

## Performance Notes

- **Release mode** is ~10x faster than debug mode
- **Auto-restart** script handles crashes gracefully
- **SOL price** updates every 10 seconds
- **Database batch** flushes every 5 seconds
- **Memory usage** ~50-100MB typical

## Logs Location

- **tmux/nohup**: `logs/bot.log` and `logs/bot.error.log`
- **systemd**: `sudo journalctl -u copytrader-bot`
- **Manual run**: stdout/stderr in terminal
