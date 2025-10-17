-- Add alias column to raw_events for direct wallet name storage
-- This allows querying by wallet name without parsing JSON

ALTER TABLE raw_events 
ADD COLUMN IF NOT EXISTS alias TEXT;

-- Add index for fast alias queries
CREATE INDEX IF NOT EXISTS idx_raw_events_alias 
ON raw_events(alias) 
WHERE alias IS NOT NULL;

-- Add comment
COMMENT ON COLUMN raw_events.alias IS 'Wallet alias/name from wallets table (human-readable identifier)';
