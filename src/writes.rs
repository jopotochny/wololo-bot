use tracing::error;
use crate::structs::{Ping, WololoUser};

pub(crate) async fn create_user(pool: &sqlx::PgPool, discord_id: u64) -> Option<WololoUser> {
    let user = WololoUser {
        discord_id: discord_id as i64,
        created_at: chrono::offset::Utc::now()
    };
    let result = sqlx::query(
        "INSERT into users (discord_id, created_at) VALUES ($1, $2)",
    )
        .bind(user.discord_id)
        .bind(user.created_at.timestamp())
        .fetch_one(pool)
        .await;

    match result {
        Ok(_) => Some(user),
        Err(e) => {
            error!("Unable to get user {}: {}", discord_id, e);
            None
        },
    }

}

pub(crate) async fn create_ping(pool: &sqlx::PgPool, user_discord_id: u64, discord_channel_id: u64) -> Option<Ping> {
    let ping = Ping {
        user_discord_id: user_discord_id as i64,
        discord_channel_id: discord_channel_id as i64,
        created_at: chrono::offset::Utc::now(),
        last_notified: None
    };
    let result = sqlx::query(
        "INSERT into ping_list (discord_user_id, discord_channel_id, created_at) VALUES ($1, $2, $3) RETURNING *",
    )
        .bind(ping.user_discord_id)
        .bind(ping.discord_channel_id)
        .bind(ping.created_at.timestamp())
        .fetch_one(pool)
        .await;

    match result {
        Ok(_) => Some(ping),
        Err(e) => {
            error!("Unable to create ping for user {} channel {}: {}", user_discord_id, discord_channel_id, e);
            None
        },
    }
}


pub(crate) async fn delete_ping(pool: &sqlx::PgPool, ping: Ping) -> bool {

    let result = sqlx::query(
        "DELETE FROM ping_list WHERE discord_user_id = $1 AND discord_channel_id = $2 RETURNING *",
    )
        .bind(ping.user_discord_id)
        .bind(ping.discord_channel_id)
        .fetch_one(pool)
        .await;

    match result {
        Ok(_) => true,
        Err(e) => {
            error!("Unable to delete ping for user {} channel {}: {}", ping.user_discord_id, ping.discord_channel_id, e);
            false
        },
    }
}


pub(crate) async fn update_notified_at_for_ping(pool: &sqlx::PgPool, ping: Ping) -> bool {
    let now = chrono::offset::Utc::now();
    let result = sqlx::query(
        "UPDATE ping_list SET notified_at = $3 WHERE discord_user_id = $1 AND discord_channel_id = $2  RETURNING *",
    )
        .bind(ping.user_discord_id)
        .bind(ping.discord_channel_id)
        .bind(now.timestamp())
        .fetch_one(pool)
        .await;
    match result {
        Ok(_) => true,
        Err(e) => {
            error!("Unable to update last_notified of ping for user {} channel {}: {}", ping.user_discord_id, ping.discord_channel_id, e);
            false
        },
    }

}