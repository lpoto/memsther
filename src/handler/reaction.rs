use deadpool_postgres::Pool;
use serenity::{
    model::prelude::{GuildId, Message, Reaction, UserId},
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

    let (meme_author_id, guild_id) =
        match extract_reaction_data(&ctx, &reaction).await {
            | Ok(v) => v,
            | Err(why) => {
                log::warn!("{}", why);
                return;
            }
        };
    if !validate_author_id(&meme_author_id, &reaction.user_id) {
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
    let (meme_author_id, guild_id) =
        match extract_reaction_data(&ctx, &reaction).await {
            | Ok(v) => v,
            | Err(why) => {
                log::warn!("{}", why);
                return;
            }
        };
    if !validate_author_id(&meme_author_id, &reaction.user_id) {
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
    meme_author_id: &UserId,
    reaction_author_id: &Option<UserId>,
) -> bool {
    // NOTE: ensure that the meme's author does not vote on it's
    // their own meme
    let reaction_author_id = match reaction_author_id {
        | Some(user_id) => {
            if user_id == meme_author_id {
                log::trace!("User voted on his message, not updating score");
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
        let bot_user_id = BOT_USER_ID.unwrap();
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
) -> Result<(UserId, GuildId), String> {
    let guild_id = match reaction.guild_id {
        | Some(guild_id) => guild_id,
        | None => {
            return Err(String::from("No 'guild_id' provided in the reaction"))
        }
    };

    let message =
        reaction.message(&ctx.http).await.map_err(|err| err.to_string())?;

    let author_id = get_bot_message_author_id(&message)
        .await
        .map_err(|err| err.to_string())?;

    Ok((UserId::from(author_id), guild_id))
}

async fn get_bot_message_author_id(
    message: &Message,
) -> Result<UserId, String> {
    unsafe {
        match BOT_USER_ID {
            | None => return Err(String::from("Bot user not available")),
            | Some(id) => {
                if id != message.author.id {
                    return Err(String::from("Not a memsther message"));
                }
            }
        }
    }
    let interaction = match &message.interaction {
        | Some(interaction) => interaction,
        | None => return Err(String::from("Not a memsther message")),
    };
    Ok(interaction.user.id)
}
