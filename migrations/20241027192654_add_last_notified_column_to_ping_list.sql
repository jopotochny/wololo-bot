-- Add migration script here
ALTER TABLE ping_list ADD COLUMN IF NOT EXISTS last_notified BIGINT;