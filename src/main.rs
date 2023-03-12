use std::env;

use serenity::prelude::{Client, GatewayIntents};

mod datastore;
mod handler;
mod util;

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp(None).init();

    let discord_token =
        env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN env variable");

    log::info!("Setting discord event handler ...");
    let mut client = Client::builder(
        &discord_token,
        GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_MESSAGE_REACTIONS,
    )
    .event_handler(handler::Handler::new().await)
    .await
    .expect("Error creating serenity client");

    if let Err(why) = client.start_autosharded().await {
        log::error!("Serenity client error: {:?}", why);
    }
}
