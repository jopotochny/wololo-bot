use chrono::{TimeZone, Utc};
use sqlx::{Row};
use tracing::error;
use crate::structs::WololoUser;

pub(crate) async fn get_user(pool: &sqlx::PgPool, discord_id: u64) -> Option<WololoUser> {
    let result = sqlx::query(
        "SELECT discord_id, created_at FROM users WHERE discord_id = $1",
    )
        .bind(discord_id as i64)
        .fetch_one(pool)
        .await;

    match result {
        Ok(row) => Some(WololoUser{ discord_id: row.get("discord_id"), created_at: Utc.timestamp_opt(row.get("created_at"), 0).unwrap()}),
        Err(e) => {
            error!("Unable to get user {}: {}", discord_id, e);
            None
        },
    }

}