-- Add migration script here
ALTER TABLE IF EXISTS message_children ADD COLUMN parent_channel_id  BIGINT NOT NULL;
ALTER TABLE IF EXISTS message_children ADD COLUMN child_channel_id  BIGINT NOT NULL;
