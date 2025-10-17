# LK Liquidity Wallet Analysis

## Overview
**Wallet**: `39azUYFWPz3VHgKCf3VChUwbpURdCHRxjWVowf5jUJjg`  
**Alias**: LK Liquidity  
**Activity Type**: WITHDRAW operations

## What This Wallet Does

LK Liquidity is **NOT a trader** - it's an admin/protocol wallet that performs **liquidity withdrawals** when Pump.fun bonding curves complete.

### From the Pump.fun IDL:
```json
{
  "name": "withdraw",
  "docs": ["Allows the admin to withdraw liquidity for a migration once the bonding curve completes"],
  "accounts": [...],
  "args": []
}
```

### Process Flow:
1. A Pump.fun token reaches its bonding curve completion
2. The bonding curve triggers a migration to Raydium (DEX)
3. LK Liquidity wallet calls the `withdraw` instruction
4. Liquidity is extracted from the bonding curve for migration

## Transaction Characteristics

From database analysis:
- **93 WITHDRAW transactions** captured
- **Very small fees**: avg 0.000057 SOL (5.7e-05)
- **No balance changes** recorded (liquidity withdrawal, not SOL transfer)
- **Consistent pattern**: Same operation repeated across many tokens

## Discriminator

The WITHDRAW instruction discriminator is:
```rust
const DISCRIMINATOR_WITHDRAW: [u8; 8] = [0xb7, 0x12, 0x46, 0x9c, 0x94, 0x6d, 0xa1, 0x22];
```

## Why This Matters

### For Copy Trading:
❌ **Do NOT copy** LK Liquidity trades - they're not trades!
- These are admin operations, not market trades
- No profit/loss to analyze
- Not indicative of trading strategy

### For Analytics:
✅ **Useful for market analysis**:
- Indicates which tokens completed their bonding curves
- Shows migration timing from Pump.fun → Raydium
- Can identify "successful" token launches

## Decoder Update

We've updated the decoder to properly recognize WITHDRAW as a separate action type:
- Action enum now includes `Action::Withdraw`
- WITHDRAW events properly tagged (no longer UNKNOWN)
- Can filter out in analytics: `WHERE action IN ('BUY', 'SELL')` excludes WITHDRAW

## Recommendation

For your copy trading bot:
1. **Skip WITHDRAW** actions in analytics
2. **Focus on BUY/SELL** from actual trader wallets
3. **Use WITHDRAW data** to identify tokens that "graduated" to Raydium
4. Consider tracking which tokens your traders bought that later got WITHDRAW events (successful picks!)

## SQL to Verify

Check WITHDRAW events:
```sql
SELECT 
    COUNT(*) as withdraw_count,
    MIN(created_at) as first_withdraw,
    MAX(created_at) as last_withdraw
FROM raw_events 
WHERE action = 'WITHDRAW' 
  AND wallet = '39azUYFWPz3VHgKCf3VChUwbpURdCHRxjWVowf5jUJjg';
```

Find tokens that reached completion:
```sql
SELECT DISTINCT mint, created_at
FROM raw_events
WHERE action = 'WITHDRAW'
ORDER BY created_at DESC
LIMIT 10;
```
