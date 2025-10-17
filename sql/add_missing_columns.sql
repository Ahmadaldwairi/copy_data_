-- Add missing columns to raw_events table for complete event tracking

-- 1. Add block_time for chain timestamp (separate from local receive time)
ALTER TABLE raw_events 
ADD COLUMN IF NOT EXISTS block_time TIMESTAMPTZ;

-- 2. Add recv_time_ns for local receive timestamp (for latency analysis)
ALTER TABLE raw_events 
ADD COLUMN IF NOT EXISTS recv_time_ns BIGINT;

-- 3. Add ix_index to track which instruction in the transaction
ALTER TABLE raw_events 
ADD COLUMN IF NOT EXISTS ix_index INT;

-- 4. Add decode status tracking
ALTER TABLE raw_events 
ADD COLUMN IF NOT EXISTS decode_ok BOOLEAN NOT NULL DEFAULT TRUE;

ALTER TABLE raw_events 
ADD COLUMN IF NOT EXISTS decode_err TEXT;

-- 5. Add index on (action, recv_time_ns) for fast time-based queries
CREATE INDEX IF NOT EXISTS idx_events_action_recv_time 
ON raw_events(action, recv_time_ns) 
WHERE recv_time_ns IS NOT NULL;

-- 6. Add index on decode errors for troubleshooting
CREATE INDEX IF NOT EXISTS idx_events_decode_errors 
ON raw_events(decode_ok, decode_err) 
WHERE decode_ok = FALSE;

COMMENT ON COLUMN raw_events.block_time IS 'Chain timestamp from transaction metadata';
COMMENT ON COLUMN raw_events.recv_time_ns IS 'Local receive timestamp in nanoseconds for latency analysis';
COMMENT ON COLUMN raw_events.ix_index IS 'Instruction index within the transaction';
COMMENT ON COLUMN raw_events.decode_ok IS 'Whether instruction decode was successful';
COMMENT ON COLUMN raw_events.decode_err IS 'Error message if decode failed';
