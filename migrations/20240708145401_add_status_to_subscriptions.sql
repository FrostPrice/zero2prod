-- Add Status to Subscriptions
ALTER TABLE subscriptions
ADD COLUMN status TEXT NULL;