use rand::{seq::SliceRandom, thread_rng};
use reqwest::Response;
use serenity::{
    model::prelude::{
        command::{Command, CommandOptionType},
        interaction::{
            application_command::ApplicationCommandInteraction, MessageFlags,
        },
        ReactionType,
    },
    prelude::Context,
};

use crate::util;

pub fn name() -> String { String::from("gif") }
pub fn description() -> String { String::from("Send a gif") }

#[derive(serde::Deserialize)]
struct GifResponse {
    data: Vec<Gif>,
}

#[derive(serde::Deserialize)]
struct Gif {
    url: String,
}

/// Register the hif command, it has a required
/// string option, which should contain some keywords, so we may find a gif.
pub async fn register(ctx: &Context) {
    log::trace!("Registering '{}' command ...", name());
    match Command::create_global_application_command(&ctx.http, |command| {
        command.name(name()).description(description()).create_option(
            |option| {
                option
                    .name("keywords")
                    .description("The keywords to find the gif by")
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

pub async fn handle_command(
    ctx: Context,
    command: ApplicationCommandInteraction,
    tenor_token: &str,
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
    let keywords = match command
        .data
        .options
        .iter()
        .find(|option| option.kind == CommandOptionType::String)
    {
        | Some(value) => match &value.value {
            | Some(url) => url.to_string(),
            | None => {
                log::warn!("Received gif command with no string option");
                return;
            }
        },
        | None => {
            log::warn!("Received gif command with no string option");
            return;
        }
    };
    let tenor_url = get_tenor_url(keywords, tenor_token);
    match get_gif_url(tenor_url).await {
        | Ok(url) => respond_with_gif_url(&ctx, &command, url.as_str()).await,
        | Err(why) => {
            log::warn!("Failed to fetch a gif: {}", why);
            respond_on_error(&ctx, &command).await;
        }
    };
}

async fn respond_with_gif_url(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    url: &str,
) {
    match command
        .create_interaction_response(&ctx.http, |response| {
            response.interaction_response_data(|message| message.content(url))
        })
        .await
    {
        | Ok(_) => {
            log::trace!("Successfully responded with a valid gif");
            match command.get_interaction_response(&ctx.http).await {
                | Err(_) => (),
                | Ok(message) => {
                    // NOTE: On successful meme gif, react to the sent
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
        | Err(why) => log::warn!("Failed to respond with a valid gif: {}", why),
    };
}

async fn respond_on_error(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    match command
        .create_interaction_response(&ctx.http, |response| {
            response.interaction_response_data(|message| {
                message
                    .content("Could not find any gifs")
                    .flags(MessageFlags::EPHEMERAL)
            })
        })
        .await
    {
        | Ok(_) => {
            log::trace!("Successfully responded on gif error");
        }
        | Err(why) => log::warn!("Failed to respond to gif error: {}", why),
    };
}

async fn get_gif_url(tenor_url: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let res: Response =
        client.get(tenor_url).send().await.map_err(|err| err.to_string())?;
    let gif_data: GifResponse =
        res.json().await.map_err(|err| err.to_string())?;

    if gif_data.data.len() == 0 {
        return Err(String::from("Found no gif results"));
    }
    Ok(gif_data.data.choose(&mut thread_rng()).unwrap().url.clone())
}

fn get_tenor_url(keywords: String, token: &str) -> String {
    format!(
        "https://api.giphy.com/v1/gifs/search?q={}&api_key={}&limit=25&lang=en",
        keywords, token
    )
}