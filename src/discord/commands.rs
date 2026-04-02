use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseMessage, UnavailableGuild,
};

pub async fn register_commands(ctx: &Context, guilds: &[UnavailableGuild]) {
    let commands = vec![
        CreateCommand::new("stop").description("Stop the current task"),
        CreateCommand::new("see").description("Get a screenshot"),
        CreateCommand::new("status").description("Get player status (health, coordinates)"),
        CreateCommand::new("quit").description("Disconnect from the current world"),
    ];

    for guild in guilds {
        if let Err(e) = guild.id.set_commands(&ctx.http, commands.clone()).await {
            eprintln!("discord: failed to register commands for guild {}: {e}", guild.id);
        } else {
            println!("discord: slash commands registered for guild {}", guild.id);
        }
    }
}

pub async fn handle_command(ctx: &Context, command: &CommandInteraction) {
    let (content, ephemeral) = match command.data.name.as_str() {
        "stop" => handle_stop().await,
        "see" => handle_see().await,
        "status" => handle_status().await,
        "quit" => handle_quit().await,
        _ => ("Unknown command.".to_string(), true),
    };

    let message = CreateInteractionResponseMessage::new()
        .content(content)
        .ephemeral(ephemeral);

    let response = CreateInteractionResponse::Message(message);

    if let Err(e) = command.create_response(&ctx.http, response).await {
        eprintln!("discord: failed to respond to command: {e}");
    }
}

async fn handle_stop() -> (String, bool) {
    // TODO: integrate with game engine
    ("Stopping current task...".to_string(), false)
}

async fn handle_see() -> (String, bool) {
    // TODO: capture and attach screenshot
    ("Screenshot capture not yet implemented.".to_string(), true)
}

async fn handle_status() -> (String, bool) {
    // TODO: read player state
    ("Status check not yet implemented.".to_string(), true)
}

async fn handle_quit() -> (String, bool) {
    // TODO: disconnect from world
    ("Disconnecting from world...".to_string(), false)
}
