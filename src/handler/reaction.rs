use deadpool_postgres::Pool;
use serenity::{
    model::prelude::{GuildId, Reaction, UserId},
    prelude::Context,
};

use crate::{datastore, util};

/// Check whether the reaction has been added to a message sent
/// by the bot, and if the reaction is either thumbs up or thumbs down.
/// Extract the userID from the message's content and increate or decrease
/// the user's score based on the added reaction.
pub async fn handle_reaction_add(
    ctx: Context,
    reaction: Reaction,
    pool: &Pool,
) {
    if !reaction.emoji.unicode_eq(util::get_thumbs_up().as_str())
        && !reaction.emoji.unicode_eq(util::get_thumbs_down().as_str())
    {
        return;
    }
    let guild_id = match reaction.guild_id {
        | Some(guild_id) => guild_id,
        | None => return,
    };

    let message = match reaction.message(&ctx.http).await {
        | Err(why) => {
            log::error!("Could not fetch message: {:?}", why);
            return;
        }
        | Ok(message) => message,
    };
    let author_id =
        match get_meme_author_id(&ctx, &guild_id, &message.content).await {
            | Err(_) => return,
            | Ok(id) => UserId(id),
        };

    match update_user_score(
        author_id,
        guild_id,
        pool,
        reaction.emoji.unicode_eq(util::get_thumbs_down().as_str()),
    )
    .await
    {
        | Err(why) => log::error!("Could not update user score: {}", why),
        | Ok(_) => log::trace!("Updated user {}'s score", author_id),
    }
}

/// Check whether the reaction has been from a message setn
/// by the bot, and if the reaction is either thumbs up or thumbs down.
/// Extract the userID from the message's content and decrease or increse
/// the user's score based on the added reaction.
pub async fn handle_reaction_remove(
    ctx: Context,
    reaction: Reaction,
    pool: &Pool,
) {
    if !reaction.emoji.unicode_eq(util::get_thumbs_up().as_str())
        && !reaction.emoji.unicode_eq(util::get_thumbs_down().as_str())
    {
        return;
    }
    let guild_id = match reaction.guild_id {
        | Some(guild_id) => guild_id,
        | None => return,
    };
    let message = match reaction.message(&ctx.http).await {
        | Err(why) => {
            log::error!("Could not fetch message: {:?}", why);
            return;
        }
        | Ok(message) => message,
    };
    let author_id =
        match get_meme_author_id(&ctx, &guild_id, &message.content).await {
            | Err(_) => return,
            | Ok(id) => UserId(id),
        };
    match update_user_score(
        author_id,
        guild_id,
        pool,
        reaction.emoji.unicode_eq(util::get_thumbs_up().as_str()),
    )
    .await
    {
        | Err(why) => log::error!("Could not update user score: {}", why),
        | Ok(_) => log::trace!("Updated user {}'s score", author_id),
    }
}

pub async fn update_user_score(
    user_id: UserId,
    guild_id: GuildId,
    pool: &Pool,
    decrement: bool,
) -> Result<(), String> {
    if decrement {
        datastore::user::decrement_score(pool, user_id, guild_id).await
    } else {
        datastore::user::increment_score(pool, user_id, guild_id).await
    }
}

/// Extract the author id from the message's content, returns
/// error when the message is not a meme message, or the id
/// could not be extracted.
pub async fn get_meme_author_id(
    ctx: &Context,
    guild_id: &GuildId,
    content: &String,
) -> Result<u64, String> {
    let s = content.trim();
    if s.len() < 40 {
        return Err(String::from("Not a meme message"));
    };
    let prefix = s[..3].to_string();
    let suffix = s[21..40].to_string();
    if !(prefix.eq("<@!") && suffix.starts_with("> just sent a meme")) {
        return Err(String::from("Not a meme message"));
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
