use chrono::{TimeZone, Utc};
use sqlx::{Error, Row};
use tracing::error;
use crate::structs::{Ping};
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

pub(crate) async fn get_ping(pool: &sqlx::PgPool, user_discord_id: u64, discord_channel_id: u64) -> Option<Ping> {
    let result = sqlx::query(
        "SELECT discord_user_id, discord_channel_id, created_at, last_notified FROM ping_list WHERE discord_user_id = $1 AND discord_channel_id = $2",
    )
        .bind(user_discord_id as i64)
        .bind(discord_channel_id as i64)
        .fetch_one(pool)
        .await;

    match result {
        Ok(row) => Some(
            Ping {
                user_discord_id: row.get("discord_user_id"),
                discord_channel_id: row.get("discord_channel_id"),
                created_at: Utc.timestamp_opt(row.get("created_at"), 0).unwrap(),
                last_notified: {
                    if let Some(last_notified) = row.get("last_notified") {
                        Some(Utc.timestamp_opt(last_notified, 0).unwrap())
                    }
                    else {
                        None
                    }
                }
            }
        ),
        Err(e) => {
            error!(
                "Unable to get ping for user {} channel {}: {}",
                user_discord_id,
                discord_channel_id,
                e
            );
            None
        },
    }

}

pub(crate) async fn get_all_pings_except_for_user(pool: &sqlx::PgPool, user_discord_id: u64, discord_channel_id: u64) -> Vec<Ping> {
    let result = sqlx::query(
        "SELECT discord_user_id, discord_channel_id, created_at, last_notified FROM ping_list WHERE discord_user_id != $1 AND discord_channel_id = $2",
    )
        .bind(user_discord_id as i64)
        .bind(discord_channel_id as i64)
        .fetch_all(pool)
        .await;
    let mut pings: Vec<Ping> = Vec::new();

    match result {
        Ok(rows) => {
            for row in rows {
                pings.push(
                    Ping {
                        user_discord_id: row.get("discord_user_id"),
                        discord_channel_id: row.get("discord_channel_id"),
                        created_at: Utc.timestamp_opt(row.get("created_at"), 0).unwrap(),
                        last_notified: {
                            if let Some(last_notified) = row.get("last_notified") {
                                Some(Utc.timestamp_opt(last_notified, 0).unwrap())
                            }
                            else {
                                None
                            }
                        }
                    }
                )
            }
            pings
        }
        Err(e) => {
            error!(
                "Unable to get ping for user {} channel {}: {}",
                user_discord_id,
                discord_channel_id,
                e
            );
            pings
        },
    }
}

pub(crate) async fn is_user_admin(pool: &sqlx::PgPool, discord_id: u64) -> Result<bool, Error> {
    let _ = sqlx::query(
        "SELECT discord_user_id FROM admins WHERE discord_user_id = $1",
    )
        .bind(discord_id as i64)
        .fetch_one(pool)
        .await?;
    Ok(true)
}