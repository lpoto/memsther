use deadpool_postgres::Pool;
use serenity::{
    model::{
        application::command::Command,
        prelude::{
            command::CommandOptionType,
            interaction::{
                application_command::ApplicationCommandInteraction,
                MessageFlags,
            },
        },
    },
    prelude::Context,
};

use crate::{datastore, util};

pub fn name() -> String { String::from("score") }
pub fn description() -> String { String::from("Get a user's score") }

/// Register the score slash command. The command has the name
/// and the description matching the values returned by `name()` and
/// `description()`. It has 1 mandatory option, containing a user.
pub async fn register(ctx: &Context) {
    log::trace!("Registering '{}' command ...", name());
    match Command::create_global_application_command(&ctx.http, |command| {
        command.name(name()).description(description()).create_option(
            |option| {
                option
                    .name("user")
                    .description("The user to get the score of")
                    .kind(CommandOptionType::User)
                    .required(true)
            },
        )
    })
    .await
    {
        | Ok(_) => log::info!("Registered '{}' slash command", name()),
        | Err(why) => {
            log::info!("Failed to register '{}' slash command: {}", name(), why)
        }
    }
}

/// Handle the meme application command. This expects the command name to match
/// the value returned from the `name()` function. Responds to the
/// provided command with the score of the user provided as the command's
/// option.
pub async fn handle_command(
    ctx: Context,
    command: ApplicationCommandInteraction,
    pool: &Pool,
) {
    log::trace!("Handling '{}' command ...", name());
    // NOTE: ensure the correct application command interaction
    // has been passes to this function, as it depends on the
    // command configuration specified in the `register` function.
    if command.data.name != name() {
        log::warn!(
            "Received command interaction for '{}' but expected '{}'",
            command.data.name,
            name()
        );
        return;
    };

    let guild_id = match command.guild_id {
        | Some(guild_id) => guild_id,
        | None => return,
    };

    let (user_id, username) =
        match util::get_user_id_from_interaction(&command).await {
            | Ok((user_id, username)) => (user_id, username),
            | Err(why) => {
                log::warn!("Failed to get user id: {}", why);
                return;
            }
        };
    let score = match datastore::user::get_score(&pool, user_id, guild_id).await
    {
        | Err(why) => {
            log::warn!("Failed to get user score: {}", why);
            return;
        }
        | Ok(score) => score,
    };
    let content = format!("**_{}_** has a score of **_{}_**", username, score);
    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response.interaction_response_data(|message| {
                message.content(content).flags(MessageFlags::EPHEMERAL)
            })
        })
        .await
    {
        log::warn!("Failed to respond to score command: {}", why);
    }
}
