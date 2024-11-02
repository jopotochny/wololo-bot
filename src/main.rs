mod queries;
mod structs;
mod writes;
mod constants;

use anyhow::Context as _;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_openai::async_openai;
use shuttle_openai::async_openai::config::OpenAIConfig;
use shuttle_runtime::SecretStore;
use tracing::{error, info};
use crate::queries::{get_all_pings_except_for_user, get_ping, get_user, is_user_admin};
use crate::structs::WololoUser;
use crate::writes::{create_admin_user, create_ping, create_user, delete_ping, update_notified_at_for_ping};
use regex::Regex;

struct Bot {
    database: sqlx::PgPool
}

const NOTIFICATION_TIMEOUT_SECONDS: u64 = 60 * 2;  // 2 MIN

async fn handle_command(command: &str, rest_of_command: Option<&str>, ctx: Context, msg: &Message, database: &sqlx::PgPool) {
    let user_discord_id = msg.author.id.get();
    let discord_channel_id = msg.channel_id.get();
    let discord_channel_name = msg.channel_id.name(&ctx.http).await.unwrap();
    match command {
        "!help" => if let Err(e) = msg.channel_id.say(&ctx.http, constants::HELP_TEXT).await {
            error!("Error sending message: {:?}", e);
        }
        "!register" =>   {
            let user = get_or_create_user(database, user_discord_id).await;
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
            let ping = get_ping(database, user_discord_id, discord_channel_id).await;
            match ping {
                Some(_) => {
                    if let Err(e) = msg.channel_id.say(&ctx.http, format!("@{} You are already signed up for game search notifications in #{}", msg.author.name, discord_channel_name)).await {
                        error!("Error sending message: {:?}", e);
                    }
                }
                None => {
                    let new_ping = create_ping(database, user_discord_id, discord_channel_id).await;
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
            let ping = get_ping(database, user_discord_id, discord_channel_id).await;
            if ping.is_some() {
                if delete_ping(database, ping.unwrap()).await {
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
            let user = get_user(database, user_discord_id).await;
            if user.is_none() {
                if let Err(e) = msg.channel_id.say(&ctx.http, format!("@{} You aren't registered in #{}, you can register using !register", msg.author.name, discord_channel_name)).await {
                    error!("Error sending message: {:?}", e);
                }
            }
            else {
                let now = chrono::offset::Utc::now();
                let pings = get_all_pings_except_for_user(database, user_discord_id, discord_channel_id).await;
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
                        let mut additional_context = "".to_string();
                        if rest_of_command.is_some() {
                            let rest_of_command_str = rest_of_command.unwrap();
                            if rest_of_command_str != "" {
                                additional_context = format!("They also said this: {rest_of_command_str}")
                            }
                        }

                        let builder = serenity::builder::CreateMessage::new().content(format!("@{} is trying to get a stack for dota in #{}. {}\n\n(You can unsubscribe from notifications in #{} by going there and typing !game-notification-off)", msg.author.name, discord_channel_name, additional_context, discord_channel_name ));
                        let result = user.direct_message(&ctx.http, builder).await;
                        if result.is_err() {
                            error!("Error sending dm to {} (id {}): {:?}", msg.author.name, user_discord_id, result.unwrap_err());
                        }
                        else {
                            if update_notified_at_for_ping(database, ping).await.is_err() {
                                error!("Unable to update notified_at for user {}", user.get())
                            }
                        }
                    }

                }
            }
        }
        "!admin" => {
            if let Some(_) = get_user(database, user_discord_id).await {
                if is_user_admin(database, user_discord_id).await.is_ok() {
                    // caller is an admin, lets add the
                    for mentioned_user in &msg.mentions {
                        if is_user_admin(database, mentioned_user.id.get()).await.is_ok() {
                            if let Err(e) = msg.channel_id.say(&ctx.http, format!("@{} {} is already an admin.", msg.author.name, mentioned_user.name)).await {
                                error!("Error sending message: {:?}", e);
                            }
                        }
                        else if create_admin_user(database, mentioned_user.id.get()).await.is_ok() {
                            if let Err(e) = msg.channel_id.say(&ctx.http, format!("@{} {} has been added as an admin.", msg.author.name, mentioned_user.name)).await {
                                error!("Error sending message: {:?}", e);
                            }
                        }
                        else {
                            if let Err(e) = msg.channel_id.say(&ctx.http, format!("@{} I was unable to add {} as an admin.", msg.author.name, mentioned_user.name)).await {
                                error!("Error sending message: {:?}", e);
                            }
                        }
                    }

                }
                else {
                    if let Err(e) = msg.channel_id.say(&ctx.http, format!("@{} You are not an admin.", msg.author.name)).await {
                        error!("Error sending message: {:?}", e);
                    }
                }
            }
        }
        _ => {}
    }
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
        let stripped_content = msg.content.trim();

        let user_discord_id = msg.author.id.get();
        let discord_channel_id = msg.channel_id.get();
        let discord_channel_name = msg.channel_id.name(&ctx.http).await.unwrap();
        // don't respond to bots
        if msg.author.bot {
            return;
        }
        info!("Received message from discord user {} in channel {} ({})", user_discord_id, discord_channel_id, discord_channel_name);
        // check commands first
        let command_regex = Regex::new(r"^(!\S*)(.*)").unwrap();
        if let Some(captures) = command_regex.captures(stripped_content) {
            if let Some(command) = captures.get(1) {
                let mut rest_of_command = None;
                // if they have other text after the command we may want it, so get it
                if let Some(rest_of_command_match) = captures.get(2) {
                    rest_of_command = Some(rest_of_command_match.as_str().trim())
                }

                handle_command(command.as_str(), rest_of_command, ctx, &msg, &self.database).await;
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
