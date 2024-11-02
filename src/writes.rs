use sqlx::Error;
use tracing::error;
use crate::structs::{AdminUser, ParentMessageChildMessage, Ping, WololoUser};

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


pub(crate) async fn update_notified_at_for_ping(pool: &sqlx::PgPool, ping: Ping) -> Result<Ping, Error> {
    let now = chrono::offset::Utc::now();
    let mut transaction = pool.begin().await?;
    sqlx::query(
        "SELECT FROM users WHERE discord_id = $1"
    ).bind(ping.user_discord_id)
        .execute(&mut *transaction)
        .await?;
    let _ = sqlx::query(
        "UPDATE ping_list SET last_notified = $3 WHERE discord_user_id = $1 AND discord_channel_id = $2  RETURNING *",
    )
        .bind(ping.user_discord_id)
        .bind(ping.discord_channel_id)
        .bind(now.timestamp())
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(Ping {
        user_discord_id: ping.user_discord_id,
        discord_channel_id: ping.discord_channel_id,
        created_at: ping.created_at,
        last_notified: Option::from(now)
    })

}

pub(crate) async fn create_admin_user(pool: &sqlx::PgPool, discord_id: u64) -> Result<AdminUser, Error> {
    let _ = sqlx::query(
        "INSERT into admins (discord_user_id) VALUES ($1) RETURNING * ",
    ).bind(discord_id as i64)
        .fetch_one(pool)
        .await?;
    Ok(AdminUser {
        discord_id: discord_id as i64
    })
}

pub(crate) async fn create_child_for_message(pool: &sqlx::PgPool, parent_msg_child_msg: ParentMessageChildMessage) -> Result<bool, Error> {
    let _ = sqlx::query(
        "INSERT into message_children (parent, child, parent_channel_id, child_channel_id) VALUES ($1, $2, $3, $4) RETURNING * ",
    ).bind(parent_msg_child_msg.parent)
        .bind(parent_msg_child_msg.child)
        .bind(parent_msg_child_msg.parent_channel_id)
        .bind(parent_msg_child_msg.child_channel_id)
        .fetch_one(pool)
        .await?;
    Ok(true)
}

pub(crate) async fn delete_child_for_message(pool: &sqlx::PgPool, parent_msg_child_msg: ParentMessageChildMessage) -> Result<bool, Error> {
    let _ = sqlx::query(
        "DELETE from message_children WHERE parent=$1 AND child=$2 RETURNING *",
    ).bind(parent_msg_child_msg.parent)
        .bind(parent_msg_child_msg.child)
        .fetch_one(pool)
        .await?;
    Ok(true)
}