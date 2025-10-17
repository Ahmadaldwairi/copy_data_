-- Add parsed pre/post balance fields for P&L accuracy
-- These are extracted from meta_json for fast queries without JSON parsing

ALTER TABLE raw_events 
ADD COLUMN IF NOT EXISTS pre_balance_sol DOUBLE PRECISION;

ALTER TABLE raw_events 
ADD COLUMN IF NOT EXISTS post_balance_sol DOUBLE PRECISION;

ALTER TABLE raw_events 
ADD COLUMN IF NOT EXISTS balance_change_sol DOUBLE PRECISION;

-- Add index for P&L queries
CREATE INDEX IF NOT EXISTS idx_raw_events_balance_change 
ON raw_events(wallet, balance_change_sol) 
WHERE balance_change_sol IS NOT NULL;

-- Add comments
COMMENT ON COLUMN raw_events.pre_balance_sol IS 'Wallet SOL balance before transaction';
COMMENT ON COLUMN raw_events.post_balance_sol IS 'Wallet SOL balance after transaction';
COMMENT ON COLUMN raw_events.balance_change_sol IS 'Net SOL balance change (post - pre)';
