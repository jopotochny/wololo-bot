-- Add migration script here
CREATE TABLE IF NOT EXISTS message_children (
    id SERIAL PRIMARY KEY NOT NULL,
    parent BIGINT NOT NULL,
    child BIGINT NOT NULL
);
