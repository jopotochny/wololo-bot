mod util;
mod queries;
mod structs;
mod writes;

use anyhow::Context as _;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::{error, info};
use crate::queries::get_user;
use crate::structs::WololoUser;
use crate::writes::create_user;

struct Bot {
    database: sqlx::PgPool
}

async fn get_or_create_user(pool: &sqlx::PgPool, discord_id: u64) -> Option<WololoUser> {
    let mut user = get_user(pool, discord_id).await;
    if user.is_none() {
        user = create_user(pool, discord_id).await;
        // we could run into an error if we for some reason have already created this user
        // pretty overkill for this but eh
        if user.is_none() {
            user = get_user(pool, discord_id).await;
        }
    }
    user

}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        let stripped_content = util::remove_whitespace(&msg.content);
        if stripped_content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!").await {
                error!("Error sending message: {:?}", e);
            }
        }
        if stripped_content == "!register" {
            let user = get_or_create_user(&self.database, msg.author.id.get()).await;
            match user {
                Some(user) => {
                    if let Err(e) = msg.channel_id.say(&ctx.http, format!("@{} I have successfully registered you, or you are already registered!", msg.author.name)).await {
                        error!("Error sending message: {:?}", e);
                    }
                }
                None => {
                    if let Err(e) = msg.channel_id.say(&ctx.http, format!("@{} I was unable to register you, try again later.", msg.author.name)).await {
                        error!("Error sending message: {:?}", e);
                    }
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
    #[shuttle_shared_db::Postgres] pool: sqlx::PgPool,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = secrets
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let bot = Bot {
        database: pool
    };
    let client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .expect("Err creating client");
    Ok(client.into())
}
