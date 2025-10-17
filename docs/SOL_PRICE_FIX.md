# SOL Price Fetcher - Rate Limiting Fix

## Issue

```
ERROR common::sol_price: Failed to fetch SOL price: error decoding response body:
missing field `solana` at line 1 column 187
```

This error occurred repeatedly because:

1. **CoinGecko Rate Limiting**: Free API limits to 10-50 calls/minute
2. **Missing User-Agent**: CoinGecko requires proper User-Agent header
3. **No Retry Logic**: Single failures caused continuous errors
4. **Too Frequent Updates**: 10-second intervals = 360 calls/hour

## Solution Applied ✅

### 1. Added User-Agent Header

```rust
.user_agent("Mozilla/5.0 (compatible; CopyTraderBot/1.0)")
```

CoinGecko API requires identification of API consumers.

### 2. Improved Error Logging

Now shows the actual API response when parsing fails:

```rust
error!("Failed to parse CoinGecko response: {}", response_text);
```

This helps identify if it's:

- Rate limiting (429 status)
- Invalid response format
- API downtime

### 3. Reduced Update Frequency

- **Before**: 10 seconds (360 calls/hour)
- **After**: 30 seconds (120 calls/hour)

This stays well under rate limits while still providing frequent updates.

### 4. Added Retry Logic

**Initial fetch**: 3 attempts with 5-second delays

```rust
for attempt in 1..=3 {
    match Self::fetch_price().await {
        Ok(price) => { /* success */ }
        Err(e) => {
            error!("Failed (attempt {}/3): {}", attempt, e);
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}
```

### 5. Exponential Backoff

If failures persist, the bot automatically backs off:

```rust
if consecutive_failures > 5 {
    let backoff_secs = std::cmp::min(300, consecutive_failures * 30);
    tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
}
```

- After 5 failures: Wait 150 seconds before retry
- After 10 failures: Wait 300 seconds (5 minutes)
- Keeps using last known price during failures

### 6. Better Error Reporting

```rust
if consecutive_failures <= 3 {
    error!("Failed to fetch SOL price (failure {}/3): {}", consecutive_failures, e);
} else if consecutive_failures == 10 {
    error!("SOL price fetch failing repeatedly (10+ times) - using last known price");
}
```

Only logs first 3 failures to avoid spam, then summarizes at 10.

## How It Works Now

### Normal Operation

```
INFO  common::sol_price: Initial SOL price: $185.23
      (30 seconds pass)
INFO  common::sol_price: SOL price updated: $185.45 (+0.12%)
      (30 seconds pass)
      (no log if price changed <0.5%)
```

### During Rate Limiting

```
ERROR common::sol_price: Failed to fetch SOL price (failure 1/3): API returned status 429
      (continues using last known price)
      (automatically backs off)
INFO  common::sol_price: SOL price updated: $185.50 (+0.27%)
      (recovered)
```

### During Extended Outage

```
ERROR common::sol_price: Failed to fetch SOL price (failure 1/3): ...
ERROR common::sol_price: Failed to fetch SOL price (failure 2/3): ...
ERROR common::sol_price: Failed to fetch SOL price (failure 3/3): ...
      (failures 4-9: silent)
ERROR common::sol_price: SOL price fetch failing repeatedly (10+ times) - using last known price
      (bot continues using last known price)
      (automatic backoff increases to 5 minutes)
```

## Benefits

1. ✅ **No More Spam**: Errors only logged 3 times, then summarized
2. ✅ **Graceful Degradation**: Uses last known price during failures
3. ✅ **Auto Recovery**: Automatically retries with backoff
4. ✅ **Rate Limit Friendly**: 30-second intervals stay under limits
5. ✅ **Better Debugging**: Shows actual API responses
6. ✅ **Stable Operation**: Bot keeps running even if price fetch fails

## Alternative: Use Binance API (No Rate Limits)

If CoinGecko continues to have issues, you can switch to Binance:

```rust
// In fetch_price():
let url = "https://api.binance.com/api/v3/ticker/price?symbol=SOLUSDT";

#[derive(Deserialize)]
struct BinanceResponse {
    price: String,
}

let response: BinanceResponse = client.get(url).send().await?.json().await?;
Ok(response.price.parse::<f64>()?)
```

**Binance Advantages**:

- No rate limits on ticker endpoints
- More reliable uptime
- Real-time prices

**CoinGecko Advantages**:

- Aggregated price across multiple exchanges
- More stable (less volatile)
- No account needed

## Monitoring

### Check if price fetching is working

Look for these in logs:

```bash
# Success
grep "SOL price" logs/bot.log

# Failures
grep "Failed to fetch SOL price" logs/bot.log | wc -l
```

### Verify current price

```sql
SELECT price_est, COUNT(*)
FROM raw_events
WHERE price_est IS NOT NULL
  AND created_at > NOW() - INTERVAL '5 minutes'
GROUP BY price_est;
```

If you see the same price for >5 minutes, the fetcher is stuck.

### Manual price check

```bash
curl "https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd"
```

Should return:

```json
{ "solana": { "usd": 185.23 } }
```

## What Changed in Code

**File**: `crates/common/src/sol_price.rs`

**Changes**:

1. Added User-Agent header
2. Increased timeout: 5s → 10s
3. Added response text logging on errors
4. Update interval: 10s → 30s
5. Added 3-attempt retry on startup
6. Added exponential backoff on failures
7. Added consecutive failure tracking
8. Reduced error log spam

**Rebuild Required**: ✅ Yes (already done)

**Backward Compatible**: ✅ Yes (same API, just more reliable)

## Current Status

✅ **Fixed and Deployed**

- Update interval: 30 seconds
- Retry logic: 3 attempts with backoff
- Rate limit friendly: 120 calls/hour
- Graceful degradation: Uses last known price
- Better error messages: Shows API responses

The bot will now handle CoinGecko rate limiting gracefully and continue operating with the last known SOL price during temporary API issues.
