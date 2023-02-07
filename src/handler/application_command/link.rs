use serenity::{
    model::{
        application::command::Command,
        prelude::{
            command::CommandOptionType,
            interaction::{
                application_command::ApplicationCommandInteraction,
                MessageFlags,
            },
            ReactionType,
        },
    },
    prelude::Context,
};

use crate::util;

pub fn name() -> String { String::from("link") }
pub fn description() -> String { String::from("Send a link") }

/// Register the meme link command, it has a required
/// string option, which should contain a valid url.
pub async fn register(ctx: &Context) {
    log::trace!("Registering '{}' command ...", name());
    match Command::create_global_application_command(&ctx.http, |command| {
        command.name(name()).description(description()).create_option(
            |option| {
                option
                    .name("link")
                    .description("The link to be sent")
                    .kind(CommandOptionType::String)
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

/// Handle the link application command. This expects the command name to match
/// the value returned from the `name()` function. Responds to the
/// provided command with the provided attachment and content, also
/// mentions the user who used the command.
pub async fn handle_command(
    ctx: Context,
    command: ApplicationCommandInteraction,
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
    let url = match command
        .data
        .options
        .iter()
        .find(|option| option.kind == CommandOptionType::String)
    {
        | Some(value) => match &value.value {
            | Some(url) => url.to_string(),
            | None => {
                log::warn!("Received link command with no string option");
                return;
            }
        },
        | None => {
            log::warn!("Received link command with no string option");
            return;
        }
    };
    if !util::is_url(url.as_str()) {
        log::warn!("Received invalid link");
        respond_to_invalid_url(&ctx, &command, url.as_str()).await;
        return;
    }
    respond_to_valid_url(&ctx, &command, url.as_str()).await;
}

async fn respond_to_invalid_url(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    url: &str,
) {
    match command
        .create_interaction_response(&ctx.http, |response| {
            response.interaction_response_data(|message| {
                message
                    .content(format!("_{}_  is not a valid link", url))
                    .flags(MessageFlags::EPHEMERAL)
            })
        })
        .await
    {
        | Ok(_) => {
            log::trace!("Successfully responded to invalid link");
        }
        | Err(why) => log::warn!("Failed to respond to invalid link: {why}"),
    };
}

async fn respond_to_valid_url(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    url: &str,
) {
    let user_id = command.user.id;
    match command
        .create_interaction_response(&ctx.http, |response| {
            response.interaction_response_data(|message| {
                message
                    .content(format!("<@!{user_id}> just sent a link: {url}"))
            })
        })
        .await
    {
        | Ok(_) => {
            log::trace!("Successfully responded with a valid link");
            match command.get_interaction_response(&ctx.http).await {
                | Err(_) => (),
                | Ok(message) => {
                    // NOTE: On successful meme response, react to the sent
                    // message with thumbs up and thumbs down.
                    for reaction in
                        vec![util::get_thumbs_up(), util::get_thumbs_down()]
                            .iter()
                    {
                        if let Err(why) = message
                            .react(
                                &ctx.http,
                                ReactionType::Unicode(reaction.to_string()),
                            )
                            .await
                        {
                            log::warn!("Error when reaction to meme: {why}");
                        }
                    }
                }
            };
        }
        | Err(why) => log::warn!("Failed to respond to a valid link: {}", why),
    };
}
