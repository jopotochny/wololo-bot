use tracing::error;
use crate::structs::WololoUser;

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
