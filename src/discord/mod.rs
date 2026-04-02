use std::collections::HashSet;

use serenity::all::{
    ChannelId, Client, Context, EventHandler, GatewayIntents, Interaction, Message, Ready,
};
use serenity::async_trait;
use tokio::task::JoinHandle;

use crate::settings::SharedConfig;

mod commands;

struct Handler {
    log_channel_id: ChannelId,
    allowed_channel_ids: HashSet<ChannelId>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("discord: bot connected as {}", ready.user.name);
        commands::register_commands(&ctx, &ready.guilds).await;
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        if msg.channel_id == self.log_channel_id {
            return;
        }
        if !self.allowed_channel_ids.contains(&msg.channel_id) {
            return;
        }
        let bot_id = ctx.cache.current_user().id;
        if !msg.mentions_user_id(bot_id) {
            return;
        }
        // Strip the mention(s) and trim whitespace to get the plain text
        let content = msg
            .content
            .replace(&format!("<@{bot_id}>"), "")
            .replace(&format!("<@!{bot_id}>"), "")
            .trim()
            .to_string();

        println!("discord: mention from {}: {content}", msg.author.name);

        if let Err(e) = msg.reply(&ctx.http, format!("You said: {content}")).await {
            eprintln!("discord: failed to reply to mention: {e}");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            if command.channel_id == self.log_channel_id {
                return;
            }
            if !self.allowed_channel_ids.contains(&command.channel_id) {
                return;
            }
            commands::handle_command(&ctx, &command).await;
        }
    }
}

/// Check config and optionally start the Discord bot.
/// Returns `None` if Discord config is incomplete, `Some(JoinHandle)` if the bot was spawned.
pub fn maybe_start_bot(config: SharedConfig) -> Option<JoinHandle<()>> {
    let state = config.lock().unwrap();
    let discord = &state.config.discord;

    if discord.bot_token.is_empty()
        || discord.admin_channel_id.is_empty()
        || discord.log_channel_id.is_empty()
    {
        println!("discord: config incomplete, skipping bot launch");
        return None;
    }

    let token = discord.bot_token.clone();

    let admin_channel_id: u64 = discord.admin_channel_id.parse().ok()?;
    let log_channel_id: u64 = discord.log_channel_id.parse().ok()?;
    let user_channel_ids: Vec<u64> = discord
        .user_channel_ids
        .iter()
        .filter_map(|s| s.parse().ok())
        .collect();

    drop(state);

    let mut allowed = HashSet::new();
    allowed.insert(ChannelId::new(admin_channel_id));
    for &id in &user_channel_ids {
        allowed.insert(ChannelId::new(id));
    }

    let handler = Handler {
        log_channel_id: ChannelId::new(log_channel_id),
        allowed_channel_ids: allowed,
    };

    let handle = tokio::spawn(async move {
        if let Err(e) = run_bot(token, handler).await {
            eprintln!("discord: bot error: {e}");
        }
    });

    println!("discord: bot launched");
    Some(handle)
}

async fn run_bot(token: String, handler: Handler) -> Result<(), serenity::Error> {
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await?;

    client.start().await
}
