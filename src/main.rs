use std::path::PathBuf;

use clap::Parser;
use serenity::prelude::{Client, GatewayIntents};

mod configuration;
mod datastore;
mod handler;
mod util;

#[derive(Parser)]
struct Cli {
    config: PathBuf,
}

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp(None).init();

    let args = Cli::parse();
    let config = configuration::Configuration::parse(args.config);

    log::info!("Setting discord event handler ...");
    let mut client = Client::builder(
        &config.discord.token,
        GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_MESSAGE_REACTIONS,
    )
    .event_handler(handler::Handler::new(config).await)
    .await
    .expect("Error creating serenity client");

    if let Err(why) = client.start_autosharded().await {
        log::error!("Serenity client error: {:?}", why);
    }
}
