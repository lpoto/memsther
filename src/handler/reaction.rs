use deadpool_postgres::Pool;
use serenity::{
    model::prelude::{GuildId, Reaction, UserId},
    prelude::Context,
};

use super::BOT_USER_ID;
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

    let (meme_message_author_id, meme_author_id, guild_id, _) =
        match extract_reaction_data(&ctx, &reaction).await {
            | Ok(v) => v,
            | Err(why) => {
                log::warn!("{}", why);
                return;
            }
        };
    if !validate_author_id(
        &meme_message_author_id,
        &meme_author_id,
        &reaction.user_id,
    ) {
        return;
    }

    match update_user_score(
        meme_author_id,
        guild_id,
        pool,
        reaction.emoji.unicode_eq(util::get_thumbs_down().as_str()),
    )
    .await
    {
        | Err(why) => log::error!("Could not update user score: {}", why),
        | Ok(_) => log::trace!("Updated user {}'s score", meme_author_id),
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
    let (meme_message_author_id, meme_author_id, guild_id, _) =
        match extract_reaction_data(&ctx, &reaction).await {
            | Ok(v) => v,
            | Err(why) => {
                log::warn!("{}", why);
                return;
            }
        };
    if !validate_author_id(
        &meme_message_author_id,
        &meme_author_id,
        &reaction.user_id,
    ) {
        return;
    }

    match update_user_score(
        meme_author_id,
        guild_id,
        pool,
        reaction.emoji.unicode_eq(util::get_thumbs_up().as_str()),
    )
    .await
    {
        | Err(why) => log::error!("Could not update user score: {}", why),
        | Ok(_) => log::trace!("Updated user {}'s score", meme_author_id),
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

fn validate_author_id(
    meme_message_author_id: &UserId,
    meme_author_id: &UserId,
    reaction_author_id: &Option<UserId>,
) -> bool {
    // NOTE: ensure that the meme's author does not vote on it's
    // their own meme
    let reaction_author_id = match reaction_author_id {
        | Some(user_id) => {
            if user_id == meme_author_id {
                log::trace!("User voted on his meme, not updating score");
                return false;
            };
            user_id
        }
        | None => {
            log::trace!("No user id found in the reaction, not updating score");
            return false;
        }
    };
    unsafe {
        let bot_user_id = match BOT_USER_ID {
            | Some(id) => id,
            | None => {
                log::trace!("No bot user available, cannot update score");
                return false;
            }
        };
        if bot_user_id != *meme_message_author_id {
            log::trace!(
                "Bot is not author of the meme message, cannot update score"
            );
            return false;
        }
        if bot_user_id == *reaction_author_id {
            log::trace!(
                "Bot is the author of the reaction, not updating score"
            );
            return false;
        }
    }
    true
}

async fn extract_reaction_data(
    ctx: &Context,
    reaction: &Reaction,
) -> Result<(UserId, UserId, GuildId, String), String> {
    let guild_id = match reaction.guild_id {
        | Some(guild_id) => guild_id,
        | None => {
            return Err(String::from("No 'guild_id' provided in the reaction"))
        }
    };

    let message =
        reaction.message(&ctx.http).await.map_err(|err| err.to_string())?;

    let author_id = get_meme_author_id(&ctx, &guild_id, &message.content)
        .await
        .map_err(|err| err.to_string())?;

    Ok((message.author.id, UserId::from(author_id), guild_id, message.content))
}

/// Extract the author id from the message's content, returns
/// error when the message is not a meme message, or the id
/// could not be extracted.
async fn get_meme_author_id(
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

async fn convert_user_id(
    ctx: &Context,
    guild_id: &GuildId,
    user_id: &str,
) -> Result<u64, String> {
    let user_id = user_id.parse::<u64>().map_err(|err| err.to_string())?;

    guild_id.member(&ctx.http, user_id).await.map_err(|err| err.to_string())?;
    Ok(user_id)
}
