-- Wrap the whole migration in a transaction to make sure
-- it succeeds or fails atomically.
-- `sqlx` does not do it automatically for us.
BEGIN;
-- Backfill `status` for historical entries
UPDATE subscriptions
SET status = 'confirmed'
WHERE status IS NULL;
-- Make `status` not nullable
ALTER TABLE subscriptions
ALTER COLUMN status
SET NOT NULL;
COMMIT;