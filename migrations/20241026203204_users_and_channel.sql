-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY NOT NULL,
    discord_id BIGINT NOT NULL UNIQUE,
    created_at BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS ping_list (
    id SERIAL PRIMARY KEY NOT NULL,
    discord_channel_id BIGINT NOT NULL,
    discord_user_id BIGINT NOT NULL REFERENCES users (discord_id) ,
    created_at BIGINT NOT NULL
);