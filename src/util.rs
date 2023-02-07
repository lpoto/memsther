use regex::Regex;
use serenity::{
    model::prelude::{
        interaction::application_command::{
            ApplicationCommandInteraction, CommandDataOptionValue,
        },
        GuildId, Message, UserId,
    },
    prelude::Context,
};

use crate::handler::BOT_USER_ID;

pub fn get_thumbs_up() -> String { String::from("👍") }

pub fn get_thumbs_down() -> String { String::from("👎") }

pub fn is_url(url: &str) -> bool {
    let re = Regex::new(
        r"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,4}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)"
    ).unwrap();
    re.is_match(url)
}

pub async fn get_user_id_from_interaction(
    command: &ApplicationCommandInteraction,
) -> Result<(UserId, String), String> {
    for option in command.data.options.iter() {
        match &option.resolved {
            | Some(CommandDataOptionValue::User(user, _)) => {
                return Ok((user.id, user.name.clone()))
            }
            | _ => return Err(String::from("Failed to resolve an option")),
        }
    }
    Err("No user id found in the interaction".to_string())
}

pub async fn get_bot_message_author_id(
    ctx: &Context,
    guild_id: &GuildId,
    message: &Message,
) -> Result<u64, String> {
    let s = message.content.trim();
    if s.len() < 35 {
        return Err(String::from("Not a memsther message"));
    };
    unsafe {
        let bot_user_id = match BOT_USER_ID {
            | Some(id) => id,
            | None => {
                return Err(String::from("No bot user available"));
            }
        };
        if bot_user_id != message.author.id {
            return Err(String::from("Bot is not author of the message"));
        }
    }
    let prefix = s[..3].to_string();
    let suffix = s[21..35].to_string();
    if !(prefix.eq("<@!") && suffix.starts_with("> just sent")) {
        return Err(String::from("Not a memsther message"));
    };
    convert_user_id(ctx, guild_id, &s[3..21]).await
}

pub async fn convert_user_id(
    ctx: &Context,
    guild_id: &GuildId,
    user_id: &str,
) -> Result<u64, String> {
    let user_id = user_id.parse::<u64>().map_err(|err| err.to_string())?;

    guild_id.member(&ctx.http, user_id).await.map_err(|err| err.to_string())?;
    Ok(user_id)
}
