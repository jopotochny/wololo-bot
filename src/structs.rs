#[derive(sqlx::FromRow)]
pub(crate) struct WololoUser {
    pub(crate) discord_id: i64,
    pub(crate) created_at: chrono::DateTime<chrono::Utc>
}

#[derive(sqlx::FromRow)]

pub(crate) struct AdminUser {
    pub(crate) discord_id: i64,
}
#[derive(sqlx::FromRow)]
pub(crate) struct Ping {
    pub(crate) user_discord_id: i64,
    pub(crate) discord_channel_id: i64,
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
    pub(crate) last_notified: Option<chrono::DateTime<chrono::Utc>>
}

#[derive(sqlx::FromRow)]
pub(crate) struct ParentMessageChildMessage {
    pub(crate) parent: i64,
    pub(crate) parent_channel_id: i64,
    pub(crate) child: i64,
    pub(crate) child_channel_id: i64,
}