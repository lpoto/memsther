use std::{fs, path::PathBuf};

use clap::Parser;
use serenity::prelude::{Client, GatewayIntents};

mod datastore;
mod handler;
mod util;

#[derive(Parser)]
struct Cli {
    config: PathBuf,
}

#[derive(serde::Deserialize)]
struct Discord {
    token: String,
}

#[derive(serde::Deserialize)]
struct Configuration {
    discord: Discord,
    postgres: datastore::Configuration,
}

impl Configuration {
    /// Parse the Conguration object from the
    /// toml file at the given path and panic
    /// when the file could not be read or parsed.
    fn parse(path: PathBuf) -> Configuration {
        let contents = match fs::read_to_string(path) {
            | Ok(c) => c,
            | Err(e) => panic!("Could not read config: {}", e),
        };
        match toml::from_str(&contents) {
            | Ok(c) => c,
            | Err(e) => panic!("Could not parse config: {}", e),
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp(None).init();

    let args = Cli::parse();
    let config = Configuration::parse(args.config);

    let datastore = datastore::Datastore::new(config.postgres);
    datastore.migrate().await;

    log::info!("Setting discord event handler ...");
    let mut client = Client::builder(
        &config.discord.token,
        GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_MESSAGE_REACTIONS,
    )
    .event_handler(handler::Handler::new(datastore))
    .await
    .expect("Error creating serenity client");

    if let Err(why) = client.start_autosharded().await {
        log::error!("Serenity client error: {:?}", why);
    }
}
