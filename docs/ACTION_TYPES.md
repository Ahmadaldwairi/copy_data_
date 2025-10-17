# Pump.fun Action Types Reference

Complete guide to all action types captured by the data collection bot and their meaning for analytics.

## Trading Actions (Use for Copy Trading Analytics)

### 🟢 BUY
**What it is**: User purchases tokens from a bonding curve  
**Who**: Traders, investors  
**Analytics Value**: ⭐⭐⭐⭐⭐ **CRITICAL**
- Primary signal for copy trading
- Track entry points
- Calculate position sizes
- Identify bullish sentiment

**Data Fields**:
- `amount_in`: Token amount bought (u64)
- `max_sol_cost`: Max SOL willing to spend
- `mint`: Token being bought
- `balance_change_sol`: Actual SOL spent (negative)

**SQL for Analytics**:
```sql
SELECT 
    alias,
    COUNT(*) as buy_count,
    AVG(ABS(balance_change_sol)) as avg_buy_size_sol,
    SUM(ABS(balance_change_sol)) as total_spent_sol
FROM raw_events
WHERE action = 'BUY'
GROUP BY alias
ORDER BY total_spent_sol DESC;
```

---

### 🔴 SELL
**What it is**: User sells tokens back to bonding curve  
**Who**: Traders, investors  
**Analytics Value**: ⭐⭐⭐⭐⭐ **CRITICAL**
- Exit signals
- Profit taking behavior
- Loss cutting patterns
- Calculate hold times

**Data Fields**:
- `amount_in`: Token amount sold (u64)
- `min_sol_output`: Minimum SOL expected
- `mint`: Token being sold
- `balance_change_sol`: SOL received (positive)

**SQL for Analytics**:
```sql
SELECT 
    alias,
    COUNT(*) as sell_count,
    AVG(balance_change_sol) as avg_sell_size_sol,
    SUM(balance_change_sol) as total_received_sol
FROM raw_events
WHERE action = 'SELL'
GROUP BY alias;
```

---

### ✨ CREATE
**What it is**: Launch of a new token on Pump.fun  
**Who**: Token creators, projects  
**Analytics Value**: ⭐⭐⭐ **MEDIUM**
- Identify token creators in your tracked wallets
- Track if created tokens become successful
- Correlation: Do traders who create tokens perform better?

**Data Fields**:
- `mint`: The newly created token address (important!)
- No amounts (it's a creation, not a trade)
- `name`, `symbol`, `uri` (in instruction args, not in our DB)

**SQL for Analytics**:
```sql
-- Find wallets that create tokens
SELECT alias, COUNT(*) as tokens_created
FROM raw_events
WHERE action = 'CREATE'
GROUP BY alias;

-- Track success of created tokens (did others buy them?)
SELECT 
    creator.mint,
    creator.alias as creator,
    COUNT(DISTINCT buyer.wallet) as unique_buyers,
    SUM(ABS(buyer.balance_change_sol)) as total_volume
FROM raw_events creator
LEFT JOIN raw_events buyer 
    ON creator.mint = buyer.mint 
    AND buyer.action = 'BUY'
WHERE creator.action = 'CREATE'
GROUP BY creator.mint, creator.alias;
```

---

## Admin/Protocol Actions (Skip for Trading Analytics)

### 💧 WITHDRAW
**What it is**: Protocol extracts liquidity when bonding curve completes  
**Who**: LK Liquidity (admin wallet), Pump.fun protocol  
**Analytics Value**: ⭐⭐ **LOW for trading, HIGH for market intel**
- NOT trading activity
- Indicates token "graduated" to Raydium
- Shows successful token launches

**Use Case**:
```sql
-- Find tokens your traders bought that later graduated
SELECT 
    buy.alias as trader,
    buy.mint,
    buy.created_at as bought_at,
    withdraw.created_at as graduated_at,
    (withdraw.created_at - buy.created_at) as time_to_graduation
FROM raw_events buy
JOIN raw_events withdraw 
    ON buy.mint = withdraw.mint
WHERE buy.action = 'BUY' 
  AND withdraw.action = 'WITHDRAW'
ORDER BY time_to_graduation;
```

**Filter Out**:
```sql
-- Clean trading data query
SELECT * FROM raw_events 
WHERE action IN ('BUY', 'SELL', 'CREATE')
  AND alias != 'LK Liquidity';  -- Extra safety
```

---

### ⚙️ INITIALIZE
**What it is**: Creates the global Pump.fun program state  
**Who**: Pump.fun developers (one-time at deployment)  
**Analytics Value**: ⭐ **NONE**
- Rare/one-time event
- Program initialization only
- No trading relevance

**Action**: Always filter out
```sql
WHERE action NOT IN ('INITIALIZE', 'SET_PARAMS')
```

---

### 🔧 SET_PARAMS
**What it is**: Updates global Pump.fun parameters  
**Who**: Pump.fun admin  
**Analytics Value**: ⭐ **NONE for trading**
- Protocol parameter changes
- Fee adjustments
- No trading relevance

**Parameters Updated** (from IDL):
- `feeRecipient`: Where fees go
- `initialVirtualTokenReserves`: Bonding curve params
- `initialVirtualSolReserves`: Bonding curve params
- `tokenTotalSupply`: Default supply
- `feeBasisPoints`: Fee percentage

**Action**: Always filter out

---

### ❓ UNKNOWN
**What it is**: Unrecognized discriminator  
**Who**: Potential new Pump.fun instructions  
**Analytics Value**: ⭐ **INVESTIGATION NEEDED**

**Why it appears**:
1. Pump.fun added new instructions (check for updates)
2. Corrupted data (rare)
3. Non-Pump.fun instruction wrongly captured

**Action**:
- Check logs for discriminator hex values
- Investigate if count increases
- Update decoder if new instructions found

```sql
-- Monitor UNKNOWN events
SELECT 
    COUNT(*) as unknown_count,
    MIN(created_at) as first_seen,
    MAX(created_at) as last_seen
FROM raw_events
WHERE action = 'UNKNOWN';
```

---

## Analytics Query Templates

### Clean Trading Data Only
```sql
CREATE VIEW trading_events AS
SELECT *
FROM raw_events
WHERE action IN ('BUY', 'SELL')
  AND alias IS NOT NULL
  AND alias != 'LK Liquidity';
```

### Action Distribution
```sql
SELECT 
    action,
    COUNT(*) as count,
    ROUND(100.0 * COUNT(*) / SUM(COUNT(*)) OVER (), 2) as percentage
FROM raw_events
GROUP BY action
ORDER BY count DESC;
```

### Trader Activity Summary
```sql
SELECT 
    alias,
    COUNT(*) FILTER (WHERE action = 'BUY') as buys,
    COUNT(*) FILTER (WHERE action = 'SELL') as sells,
    COUNT(*) FILTER (WHERE action = 'CREATE') as creates,
    COUNT(*) FILTER (WHERE action = 'WITHDRAW') as withdraws,
    COUNT(*) FILTER (WHERE action IN ('INITIALIZE', 'SET_PARAMS')) as admin_ops,
    COUNT(*) FILTER (WHERE action = 'UNKNOWN') as unknown
FROM raw_events
WHERE alias IS NOT NULL
GROUP BY alias
ORDER BY (buys + sells) DESC;
```

---

## Summary Table

| Action | Use for Analytics? | Who Performs It | Frequency |
|--------|-------------------|-----------------|-----------|
| BUY | ✅ YES - Critical | Traders | High |
| SELL | ✅ YES - Critical | Traders | High |
| CREATE | ✅ YES - Medium | Creators | Low |
| WITHDRAW | ⚠️ Market Intel Only | Protocol (LK Liquidity) | Medium |
| INITIALIZE | ❌ NO | Pump.fun Devs | Rare |
| SET_PARAMS | ❌ NO | Pump.fun Admin | Rare |
| UNKNOWN | 🔍 Investigate | Unknown | Should be 0 |

---

## Best Practices

1. **Always filter**: Use `WHERE action IN ('BUY', 'SELL')` for trading analytics
2. **Exclude admin wallets**: `AND alias != 'LK Liquidity'`
3. **Monitor UNKNOWN**: Should be zero - investigate if it increases
4. **Use WITHDRAW for intel**: Track token graduations, but don't copy these
5. **Track CREATE separately**: Identify creators vs traders in your wallet list
