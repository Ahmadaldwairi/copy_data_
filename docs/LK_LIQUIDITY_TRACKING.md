# Tracking LK Liquidity (WITHDRAW) Activity for Market Analysis

## Purpose

LK Liquidity's WITHDRAW operations signal when tokens "graduate" from Pump.fun to Raydium. This is valuable for analyzing trader behavior before and after token maturation.

## LK Liquidity Wallet

**Address**: `39azUYFWPz3VHgKCf3VChUwbpURdCHRxjWVowf5jUJjg`  
**Alias**: `LK Liquidity`  
**Action Type**: `WITHDRAW` (liquidity extraction)

---

## Analysis Queries

### 1. Find Tokens That Graduated (WITHDRAW Events)

```sql
-- Tokens that completed their bonding curve
CREATE VIEW graduated_tokens AS
SELECT
    mint,
    MIN(created_at) as graduation_time,
    COUNT(*) as withdraw_count
FROM raw_events
WHERE action = 'WITHDRAW'
  AND wallet = '39azUYFWPz3VHgKCf3VChUwbpURdCHRxjWVowf5jUJjg'
GROUP BY mint;
```

### 2. Trader Activity BEFORE Graduation

```sql
-- Who bought tokens before they graduated?
SELECT
    t.alias as trader,
    t.mint,
    COUNT(*) FILTER (WHERE t.action = 'BUY') as buys_before_grad,
    COUNT(*) FILTER (WHERE t.action = 'SELL') as sells_before_grad,
    SUM(ABS(t.balance_change_sol)) FILTER (WHERE t.action = 'BUY') as sol_invested,
    SUM(t.balance_change_sol) FILTER (WHERE t.action = 'SELL') as sol_returned,
    MIN(t.created_at) as first_trade,
    MAX(t.created_at) as last_trade_before_grad,
    g.graduation_time
FROM raw_events t
JOIN graduated_tokens g ON t.mint = g.mint
WHERE t.action IN ('BUY', 'SELL')
  AND t.created_at < g.graduation_time  -- BEFORE graduation
  AND t.alias IS NOT NULL
  AND t.alias != 'LK Liquidity'
GROUP BY t.alias, t.mint, g.graduation_time
ORDER BY sol_invested DESC;
```

### 3. Trader Activity AFTER Graduation

```sql
-- Did traders continue trading after graduation?
-- (Note: After graduation, trades happen on Raydium, not Pump.fun)
-- This query shows if traders sold their remaining tokens on Pump.fun
SELECT
    t.alias as trader,
    t.mint,
    COUNT(*) FILTER (WHERE t.action = 'SELL') as sells_after_grad,
    SUM(t.balance_change_sol) FILTER (WHERE t.action = 'SELL') as sol_from_post_grad_sells,
    MIN(t.created_at) as first_trade_after_grad,
    g.graduation_time
FROM raw_events t
JOIN graduated_tokens g ON t.mint = g.mint
WHERE t.action = 'SELL'
  AND t.created_at > g.graduation_time  -- AFTER graduation
  AND t.alias IS NOT NULL
  AND t.alias != 'LK Liquidity'
GROUP BY t.alias, t.mint, g.graduation_time
ORDER BY sol_from_post_grad_sells DESC;
```

### 4. Time from First Buy to Graduation

```sql
-- How long did it take for tokens to graduate after our traders bought them?
SELECT
    t.alias as trader,
    t.mint,
    MIN(t.created_at) as first_buy,
    g.graduation_time,
    (g.graduation_time - MIN(t.created_at)) as time_to_graduation,
    SUM(ABS(t.balance_change_sol)) FILTER (WHERE t.action = 'BUY') as total_invested
FROM raw_events t
JOIN graduated_tokens g ON t.mint = g.mint
WHERE t.action = 'BUY'
  AND t.alias IS NOT NULL
  AND t.alias != 'LK Liquidity'
GROUP BY t.alias, t.mint, g.graduation_time
ORDER BY time_to_graduation;
```

### 5. Success Rate: Tokens Bought That Later Graduated

```sql
-- Which traders bought tokens that later graduated?
WITH trader_tokens AS (
    SELECT DISTINCT
        alias as trader,
        mint
    FROM raw_events
    WHERE action = 'BUY'
      AND alias IS NOT NULL
      AND alias != 'LK Liquidity'
),
trader_stats AS (
    SELECT
        tt.trader,
        COUNT(DISTINCT tt.mint) as total_tokens_bought,
        COUNT(DISTINCT g.mint) as graduated_tokens,
        ROUND(100.0 * COUNT(DISTINCT g.mint) / COUNT(DISTINCT tt.mint), 2) as graduation_rate
    FROM trader_tokens tt
    LEFT JOIN graduated_tokens g ON tt.mint = g.mint
    GROUP BY tt.trader
)
SELECT * FROM trader_stats
ORDER BY graduation_rate DESC, total_tokens_bought DESC;
```

### 6. Early Bird Analysis

```sql
-- Traders who bought EARLY and held until graduation
SELECT
    t.alias as trader,
    t.mint,
    MIN(t.created_at) as entry_time,
    g.graduation_time,
    (g.graduation_time - MIN(t.created_at)) as hold_duration,
    SUM(ABS(t.balance_change_sol)) FILTER (WHERE t.action = 'BUY') as invested_sol,
    SUM(t.balance_change_sol) FILTER (WHERE t.action = 'SELL' AND t.created_at < g.graduation_time) as sold_before_grad_sol,
    CASE
        WHEN SUM(t.balance_change_sol) FILTER (WHERE t.action = 'SELL' AND t.created_at < g.graduation_time) > 0
        THEN 'Sold Before Graduation'
        ELSE 'Held Through Graduation'
    END as strategy
FROM raw_events t
JOIN graduated_tokens g ON t.mint = g.mint
WHERE t.action IN ('BUY', 'SELL')
  AND t.alias IS NOT NULL
  AND t.alias != 'LK Liquidity'
GROUP BY t.alias, t.mint, g.graduation_time
ORDER BY hold_duration;
```

### 7. LK Liquidity Activity Timeline

```sql
-- When does LK Liquidity operate?
SELECT
    DATE_TRUNC('hour', created_at) as hour,
    COUNT(*) as withdrawals,
    COUNT(DISTINCT mint) as unique_tokens
FROM raw_events
WHERE action = 'WITHDRAW'
  AND wallet = '39azUYFWPz3VHgKCf3VChUwbpURdCHRxjWVowf5jUJjg'
GROUP BY DATE_TRUNC('hour', created_at)
ORDER BY hour DESC;
```

### 8. Tokens By Graduation Speed

```sql
-- How fast do tokens graduate?
WITH token_lifecycle AS (
    SELECT
        mint,
        MIN(created_at) FILTER (WHERE action = 'CREATE') as creation_time,
        MIN(created_at) FILTER (WHERE action = 'WITHDRAW') as graduation_time
    FROM raw_events
    WHERE action IN ('CREATE', 'WITHDRAW')
    GROUP BY mint
    HAVING MIN(created_at) FILTER (WHERE action = 'CREATE') IS NOT NULL
       AND MIN(created_at) FILTER (WHERE action = 'WITHDRAW') IS NOT NULL
)
SELECT
    mint,
    creation_time,
    graduation_time,
    (graduation_time - creation_time) as lifecycle_duration,
    CASE
        WHEN (graduation_time - creation_time) < INTERVAL '1 hour' THEN 'Very Fast (<1h)'
        WHEN (graduation_time - creation_time) < INTERVAL '6 hours' THEN 'Fast (<6h)'
        WHEN (graduation_time - creation_time) < INTERVAL '24 hours' THEN 'Medium (<24h)'
        ELSE 'Slow (>24h)'
    END as speed_category
FROM token_lifecycle
ORDER BY lifecycle_duration;
```

---

## Insights You Can Extract

### 1. **Trader Quality Metrics**

- What % of tokens bought by a trader later graduate?
- High graduation rate = good at picking winners

### 2. **Timing Analysis**

- Do traders buy early (close to CREATE) or late (close to WITHDRAW)?
- Early buyers might have alpha

### 3. **Hold Strategy**

- Do successful traders hold through graduation?
- Or do they sell before WITHDRAW and miss the Raydium pump?

### 4. **Token Lifecycle Patterns**

- Fast graduation (<6h) = high initial momentum
- Slow graduation (>24h) = steady accumulation

### 5. **Market Timing**

- What time of day do graduations happen?
- Correlation with trader activity timing

---

## Creating Analytics Views

```sql
-- Create a comprehensive view for easy analysis
CREATE VIEW trader_graduation_analysis AS
SELECT
    t.alias as trader,
    t.mint,
    MIN(t.created_at) FILTER (WHERE t.action = 'BUY') as first_buy_time,
    MAX(t.created_at) FILTER (WHERE t.action = 'SELL' AND t.created_at < g.graduation_time) as last_sell_before_grad,
    g.graduation_time,
    COUNT(*) FILTER (WHERE t.action = 'BUY') as buy_count,
    COUNT(*) FILTER (WHERE t.action = 'SELL' AND t.created_at < g.graduation_time) as sell_count_before_grad,
    SUM(ABS(t.balance_change_sol)) FILTER (WHERE t.action = 'BUY') as total_invested_sol,
    SUM(t.balance_change_sol) FILTER (WHERE t.action = 'SELL' AND t.created_at < g.graduation_time) as total_returned_sol,
    (SUM(t.balance_change_sol) FILTER (WHERE t.action = 'SELL' AND t.created_at < g.graduation_time)
     - SUM(ABS(t.balance_change_sol)) FILTER (WHERE t.action = 'BUY')) as realized_pnl_sol,
    CASE
        WHEN MAX(t.created_at) FILTER (WHERE t.action = 'SELL' AND t.created_at < g.graduation_time) IS NULL
        THEN 'Held Through Graduation'
        ELSE 'Sold Before Graduation'
    END as exit_strategy
FROM raw_events t
JOIN graduated_tokens g ON t.mint = g.mint
WHERE t.action IN ('BUY', 'SELL')
  AND t.alias IS NOT NULL
  AND t.alias != 'LK Liquidity'
GROUP BY t.alias, t.mint, g.graduation_time;
```

---

## Usage Example

```sql
-- Find the best "graduation traders"
SELECT
    trader,
    COUNT(DISTINCT mint) as tokens_held_to_graduation,
    AVG(total_invested_sol) as avg_investment,
    SUM(realized_pnl_sol) as total_pnl
FROM trader_graduation_analysis
WHERE exit_strategy = 'Held Through Graduation'
GROUP BY trader
ORDER BY tokens_held_to_graduation DESC;
```

---

## Note

Remember that AFTER graduation (WITHDRAW), trading happens on **Raydium (DEX)**, not Pump.fun. Our bot only tracks Pump.fun, so we can't see post-graduation trades. But we CAN see:

- Who bought early
- Who held through graduation (diamond hands 💎)
- Who sold before graduation (paper hands 📄)
- Which tokens graduated (success signals)
