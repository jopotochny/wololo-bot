-- Add migration script here
CREATE TABLE IF NOT EXISTS admins (
    id SERIAL PRIMARY KEY NOT NULL,
    discord_user_id BIGINT NOT NULL REFERENCES users (discord_id)
);


CREATE TABLE IF NOT EXISTS blacklisted_users (
    id SERIAL PRIMARY KEY NOT NULL,
    discord_user_id BIGINT NOT NULL REFERENCES users (discord_id)
);