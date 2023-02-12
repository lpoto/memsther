use deadpool_postgres::Pool;
use serenity::{
    model::prelude::{
        command::Command,
        interaction::{
            application_command::ApplicationCommandInteraction, MessageFlags,
        },
        GuildId, UserId,
    },
    prelude::Context,
};

use crate::datastore;

pub fn name() -> String { String::from("leaderboard") }
pub fn description() -> String { String::from("Show the server's leaderboard") }

/// Register the leaderboard command, it has a required
/// string option, which should contain a valid url.
pub async fn register(ctx: &Context) {
    log::trace!("Registering '{}' command ...", name());
    match Command::create_global_application_command(&ctx.http, |command| {
        command.name(name()).description(description())
    })
    .await
    {
        | Ok(_) => log::info!("Registered '{}' slash command", name()),
        | Err(why) => {
            log::info!("Failed to register '{}' slash command: {}", name(), why)
        }
    }
}

pub async fn handle_command(
    ctx: Context,
    command: ApplicationCommandInteraction,
    pool: &Pool,
) {
    log::trace!("Running '{}' command ...", name());
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
    }
    let guild_id = match command.guild_id {
        | Some(id) => id,
        | None => {
            log::trace!("No guild_id found in the leaderboard  app. command");
            return;
        }
    };

    match datastore::user::get_scores(pool, guild_id, 20).await {
        | Err(why) => {
            log::trace!("Error when fetching scores: {}", why);
            respond_no_results(ctx, command).await;
        }
        | Ok(scores) => {
            log::trace!("Fetched {} scores", scores.len());
            if scores.len() == 0 {
                respond_no_results(ctx, command).await;
                return;
            }
            respond_with_scores(ctx, guild_id, command, scores).await;
        }
    }
}

async fn respond_no_results(
    ctx: Context,
    command: ApplicationCommandInteraction,
) {
    log::trace!("Responding to a command with an empty leaderboard");
    match command
        .create_interaction_response(&ctx.http, |response| {
            response.interaction_response_data(|message| {
                message
                    .flags(MessageFlags::EPHEMERAL)
                    .content("No positive scores were found in this server")
            })
        })
        .await
    {
        | Ok(_) => log::trace!("Successfully responded with empty leaderboard"),
        | Err(why) => {
            log::warn!("Failed to respond with an empty leaderboard: {}", why)
        }
    };
}

async fn respond_with_scores(
    ctx: Context,
    guild_id: GuildId,
    command: ApplicationCommandInteraction,
    scores: Vec<(UserId, i64)>,
) {
    let mut content: Vec<String> = Vec::new();
    for (id, score) in scores.iter() {
        match guild_id.member(&ctx.http, id).await {
            | Ok(member) => {
                let name = member.display_name();
                content.push(format!("**_{}_**: {}", name, score));
            }
            | _ => (),
        }
    }
    log::trace!("Responding to a command with a leaderboard");
    let content = content.join("\n");
    let footer = format!("Showing top {} result/s", scores.len());
    match command
        .create_interaction_response(&ctx.http, |response| {
            response.interaction_response_data(|message| {
                message.embed(|e| {
                    e.footer(|f| f.text(footer))
                        .description(content)
                        .title("Leaderboard")
                })
            })
        })
        .await
    {
        | Err(why) => {
            log::warn!("Failed to respond with a leaderboard: {}", why)
        }
        | Ok(_) => log::trace!("Successfully responded with a leaderboard"),
    };
}
