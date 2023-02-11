use std::{fs, path::PathBuf};

#[derive(serde::Deserialize)]
pub struct Configuration {
    pub postgres: Postgres,
    pub discord: Discord,
    pub tenor: Tenor,
}

#[derive(serde::Deserialize)]
pub struct Discord {
    pub token: String,
}

#[derive(serde::Deserialize)]
pub struct Tenor {
    pub token: String,
}

#[derive(serde::Deserialize)]
pub struct Postgres {
    pub host: String,
    pub user: String,
    pub password: String,
    pub dbname: String,
}

impl Configuration {
    /// Parse the Conguration object from the
    /// toml file at the given path and panic
    /// when the file could not be read or parsed.
    pub fn parse(path: PathBuf) -> Configuration {
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
