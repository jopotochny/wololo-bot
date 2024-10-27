mod util;
mod queries;
mod structs;
mod writes;

use anyhow::Context as _;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_openai::async_openai;
use shuttle_openai::async_openai::config::OpenAIConfig;
use shuttle_runtime::SecretStore;
use tracing::{error, info};
use crate::queries::{get_all_pings_except_for_user, get_ping, get_user};
use crate::structs::WololoUser;
use crate::writes::{create_ping, create_user, delete_ping, update_notified_at_for_ping};

struct Bot {
    database: sqlx::PgPool
}

const NOTIFICATION_TIMEOUT_SECONDS: u64 = 60 * 2;  // 2 MIN

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
        let user_discord_id = msg.author.id.get();
        let discord_channel_id = msg.channel_id.get();
        let discord_channel_name = msg.channel_id.name(&ctx.http).await.unwrap();
        // don't respond to bots
        if msg.author.bot {
            return;
        }
        // check commands first
        match stripped_content.as_str() {
            "!hello" => if let Err(e) = msg.channel_id.say(&ctx.http, "world!").await {
                error!("Error sending message: {:?}", e);
            }
            "!register" =>   {
                let user = get_or_create_user(&self.database, user_discord_id).await;
                match user {
                    Some(_) => {
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
            "!game-notification-on" => {
                let ping = get_ping(&self.database, user_discord_id, discord_channel_id).await;
                match ping {
                    Some(_) => {
                        if let Err(e) = msg.channel_id.say(&ctx.http, format!("@{} You are already signed up for game search notifications in #{}", msg.author.name, discord_channel_name)).await {
                            error!("Error sending message: {:?}", e);
                        }
                    }
                    None => {
                        let new_ping = create_ping(&self.database, user_discord_id, discord_channel_id).await;
                        match new_ping {
                            Some(_) => {
                                if let Err(e) = msg.channel_id.say(&ctx.http, format!("@{} You are now signed up for game search notifications in #{}", msg.author.name, discord_channel_name)).await {
                                    error!("Error sending message: {:?}", e);
                                }
                            }
                            None => {
                                if let Err(e) = msg.channel_id.say(&ctx.http, format!("@{} I was unable to sign you up for game search notifications in #{}", msg.author.name, discord_channel_name)).await {
                                    error!("Error sending message: {:?}", e);
                                }
                            }
                        }
                    }
                }
            }
            "!game-notification-off" => {
                let ping = get_ping(&self.database, user_discord_id, discord_channel_id).await;
                if ping.is_some() {
                    if delete_ping(&self.database, ping.unwrap()).await {
                        if let Err(e) = msg.channel_id.say(&ctx.http, format!("@{} You have been removed from game search notifications in #{}", msg.author.name, discord_channel_name)).await {
                            error!("Error sending message: {:?}", e);
                        }
                    }
                    else {
                        if let Err(e) = msg.channel_id.say(&ctx.http, format!("@{} I was unable to remove you from game search notifications in #{}", msg.author.name, discord_channel_name)).await {
                            error!("Error sending message: {:?}", e);
                        }
                    }
                }
                else {
                    if let Err(e) = msg.channel_id.say(&ctx.http, format!("@{} You aren't signed up for game search notifications in #{}", msg.author.name, discord_channel_name)).await {
                        error!("Error sending message: {:?}", e);
                    }
                }
            }
            "!any-gamers" => {
                let user = get_user(&self.database, user_discord_id).await;
                if user.is_none() {
                    if let Err(e) = msg.channel_id.say(&ctx.http, format!("@{} You aren't registered in #{}, you can register using !register", msg.author.name, discord_channel_name)).await {
                        error!("Error sending message: {:?}", e);
                    }
                }
                else {
                    let now = chrono::offset::Utc::now();
                    let pings = get_all_pings_except_for_user(&self.database, user_discord_id, discord_channel_id).await;
                    for ping in pings {
                        let should_notify;
                        if ping.last_notified.is_some() {
                            should_notify = (now.timestamp() - ping.last_notified.unwrap().timestamp()) >= NOTIFICATION_TIMEOUT_SECONDS as i64;
                        }
                        else {
                            // last_notified is None
                            should_notify = true
                        }
                        if should_notify {
                            let user = serenity::all::UserId::new(ping.user_discord_id as u64);
                            let builder = serenity::builder::CreateMessage::new().content(format!("@{} is trying to get a stack for dota in #{}.\n\n(You can unsubscribe from notifications in #{} by going there and typing !game-notifications-off)", msg.author.name, discord_channel_name, discord_channel_name ));
                            let result = user.direct_message(&ctx.http, builder).await;
                            if result.is_err() {
                                error!("Error sending dm to {} (id {}): {:?}", msg.author.name, user_discord_id, result.unwrap_err());
                            }
                            else {
                                update_notified_at_for_ping(&self.database, ping).await;
                            }
                        }

                    }
                }
            }
            _ => {}
        }
        // otherwise maybe later we do other stuff here
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
    #[shuttle_shared_db::Postgres] pool: sqlx::PgPool,
    #[shuttle_openai::OpenAI(api_key = "{secrets.OPENAI_API_KEY}")]
    _openai: async_openai::Client<OpenAIConfig>,
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
