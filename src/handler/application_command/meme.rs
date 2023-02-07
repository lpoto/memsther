use serenity::{
    model::{
        application::command::Command,
        channel::Message,
        prelude::{
            command::CommandOptionType,
            interaction::{
                application_command::{
                    ApplicationCommandInteraction, CommandDataOptionValue,
                },
                InteractionResponseType,
            },
            ReactionType,
        },
    },
    prelude::Context,
};

use crate::util;

pub fn name() -> String { String::from("meme") }
pub fn description() -> String { String::from("Send a meme") }

/// Register the meme slash command. The command has the name
/// and the description matching the values returned by `name()` and
/// `description()`. It has 2 options, one for the text content of the meme, and
/// one for the attachment. At least one of those two should be provided.
pub async fn register(ctx: &Context) {
    log::trace!("Registering '{}' command ...", name());
    match Command::create_global_application_command(
        &ctx.http,
        |mut command| {
            for i in 0..4 {
                let attachment = if i > 0 {
                    format!("attachment{}", i)
                } else {
                    String::from("attachment")
                };
                command = command.create_option(|option| {
                    option
                        .name(attachment)
                        .description("A file containing a meme")
                        .kind(CommandOptionType::Attachment)
                        .required(i == 0)
                })
            }
            command = command
                .name(name())
                .description(description())
                .create_option(|option| {
                    option
                        .name("content")
                        .description("The optional content of the meme")
                        .kind(CommandOptionType::String)
                        .required(false)
                });
            command
        },
    )
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

    // NOTE: mention the user who sent the meme
    let txt = format!("<@!{}> just sent a meme!", command.user.id);
    let content = command
        .data
        .options
        .iter()
        .find(|option| option.kind == CommandOptionType::String)
        .map_or(txt.clone(), |content| {
            content.value.as_ref().map_or(txt.clone(), |value| {
                format!("{}\n\n{}", txt.clone(), value.to_string())
            })
        });

    defer_meme_response(&ctx, &command).await;

    match respond_with_meme(&ctx, &command, content).await {
        | Err(why) => {
            log::info!("Err when responding with meme: {:?}", why);
            remove_original_response_on_error(&ctx, &command).await;
        }
        | Ok(message) => {
            // NOTE: On successful meme response, react to the sent
            // message with thumbs up. This will immediately add  +1
            // score to the user who sent the meme.
            for reaction in
                vec![util::get_thumbs_up(), util::get_thumbs_down()].iter()
            {
                if let Err(why) = message
                    .react(
                        &ctx.http,
                        ReactionType::Unicode(reaction.to_string()),
                    )
                    .await
                {
                    log::warn!("Error when reaction to meme: {:?}", why);
                }
            }
        }
    }
}

/// Defer the response to the meme application command,
/// so the interaction does not timeout before the response
/// is sent. This is neccessary as it may take a long time
/// to uploead videos or such attachments.
async fn defer_meme_response(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    log::trace!("Deffering '{}' slash command", name());

    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::DeferredChannelMessageWithSource)
        })
        .await
    {
        log::warn!("Failed to defer an interaction: {:?}", why);
    };
}

/// Create a followup message to the meme slash command,
/// responding with the attachment and content provided in
/// the command. If this fails, it tries to delete the original
/// response (created when deferring the command).
async fn respond_with_meme(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    content: String,
) -> Result<Message, String> {
    log::trace!(
        "Responding to '{}' slash command with the provided attachment",
        name()
    );

    let mut attachments_urls: Vec<String> = Vec::new();

    for option in command.data.options.iter() {
        match &option.resolved {
            | Some(CommandDataOptionValue::Attachment(attachment)) => {
                attachments_urls.push(attachment.url.clone())
            }
            | _ => (),
        }
    }
    command
        .create_followup_message(&ctx.http, |mut message| {
            message = message.content(&content);
            for url in attachments_urls.iter() {
                message = message.add_file(url.as_str());
            }
            message
        })
        .await
        .map_err(|err| err.to_string())
}

async fn remove_original_response_on_error(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    log::trace!("Removing origin response to '{}' slash command", name());

    if let Err(why) =
        command.delete_original_interaction_response(&ctx.http).await
    {
        log::warn!(
            "Error when deleting original interaction response: {:?}",
            why
        );
    };
}
