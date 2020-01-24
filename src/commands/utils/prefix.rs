use crate::utilities::database;
use crate::utilities::database::get_database;

use serenity::framework::standard::macros::command;
use serenity::framework::standard::Args;
use serenity::framework::standard::CommandError;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[command]
#[only_in(guilds)]
#[owners_only]
#[sub_commands(get, set, clear)]
#[description("Retrieves, sets, or clears the command prefix for the current guild.")]
fn prefix(ctx: &mut Context, message: &Message) -> CommandResult {
    message
        .channel_id
        .send_message(&ctx, move |m| {
            m.embed(move |embed| {
                embed.title("Error: Invalid / No Subcommand Entered!");
                embed.description("Please use subcommand get or set to use this command.");
                embed
            })
        })
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

#[command]
#[only_in(guilds)]
#[owners_only]
#[description = "Retrieves the command prefix for the current server."]
pub fn get(ctx: &mut Context, message: &Message) -> CommandResult {
    let prefix = database::get_prefix(&message.guild_id.unwrap())?.to_string();

    let guild = match message.guild(&ctx.cache) {
        Some(guild) => guild,
        None => {
            return message
                .channel_id
                .say(&ctx.http, "Unable to get the command prefix, as the guild cannot be located.")
                .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
        }
    };

    let guild_name = &guild.read().name;

    return message
        .channel_id
        .say(&ctx.http, format!("The currently set command prefix for {} is {}.", guild_name, prefix))
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()));
}

#[command]
#[only_in(guilds)]
#[owners_only]
#[description = "Clears the current server's currently set command prefix."]
pub fn clear(ctx: &mut Context, message: &Message) -> CommandResult {
    let _ = database::clear_prefix(&message.guild_id.unwrap());

    let guild = match message.guild(&ctx.cache) {
        Some(guild) => guild,
        None => {
            return message
                .channel_id
                .say(&ctx.http, "Unable to get the command prefix, as the guild cannot be located.")
                .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
        }
    };

    let guild_name = &guild.read().name;

    return message
        .channel_id
        .say(&ctx.http, format!("The command prefix for {} has been cleared.", guild_name))
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()));
}

#[command]
#[only_in(guilds)]
#[owners_only]
#[num_args(1)]
#[description = "Sets the command prefix for the current server."]
pub fn set(ctx: &mut Context, message: &Message, args: Args) -> CommandResult {
    let connection = match get_database() {
        Ok(connection) => connection,
        Err(_) => return Ok(()),
    };

    let prefix = args.current().unwrap_or(";");

    let guild = match message.guild(&ctx.cache) {
        Some(guild) => guild,
        None => {
            return message
                .channel_id
                .say(&ctx.http, "Unable to set command prefix, as the guild cannot be located.")
                .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
        }
    };

    let guild_id = guild.read().clone().id.as_u64().to_string();
    let guild_name = guild.read().clone().name;

    let _ = connection.execute(
        "INSERT OR REPLACE INTO guild_settings (guild_id, guild_name, prefix) values (?1, ?2, ?3)",
        &[&guild_id, &guild_name, prefix],
    );

    return message
        .channel_id
        .say(&ctx.http, format!("The command prefix for {} has been set to {}.", guild_name, prefix))
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()));
}
