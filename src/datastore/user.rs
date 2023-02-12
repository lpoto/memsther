use deadpool_postgres::Pool;
use serenity::model::prelude::{GuildId, UserId};

/// Gets  score for the user identified by the provided id.
/// If there is no existing record for the user, 0 will be returned.
pub async fn get_score(
    pool: &Pool,
    id: UserId,
    guild_id: GuildId,
) -> Result<i64, String> {
    log::trace!("Fetching a user {}'s score", id);

    let client = pool.get().await.map_err(|err| err.to_string())?;
    match client
        .query_one(
            r#"
            SELECT score
            FROM "user"
            WHERE "user".id = $1 AND
                "user".guild_id = $2;
            "#,
            &[&(i64::from(id)), &(i64::from(guild_id))],
        )
        .await
    {
        | Ok(row) => Ok(row.get(0)),
        | Err(_) => Ok(0),
    }
}

/// Gets a vector of userId, score pairs, where
/// the resuls are descendingly sorted by the scores,
/// and limited by the provided limit.
/// Returns only results for the provided guildID.
pub async fn get_scores(
    pool: &Pool,
    guild_id: GuildId,
    limit: u16,
) -> Result<Vec<(UserId, i64)>, String> {
    log::trace!("Fetching top {} scores for guild: {}", limit, guild_id);
    let client = pool.get().await.map_err(|err| err.to_string())?;
    client
        .query(
            r#"
            SELECT id, score FROM "user"
            WHERE "user".guild_id = $1 AND
                "user".score > 0
            ORDER BY "user".score DESC
            LIMIT $2;
            "#,
            &[&(i64::from(guild_id)), &(i64::from(limit))],
        )
        .await
        .map_err(|err| err.to_string())
        .map(|rows| {
            // NOTE: map the rows into a vector of userId, score tuples
            rows.iter()
                .map(|row| {
                    (
                        UserId::from(row.get::<usize, i64>(0) as u64),
                        i64::from(row.get::<usize, i64>(1)),
                    )
                })
                .collect::<Vec<(UserId, i64)>>()
        })
}

/// Increment the score of the user identified by the provided
/// id by 1.
pub async fn increment_score(
    pool: &Pool,
    id: UserId,
    guild_id: GuildId,
) -> Result<(), String> {
    log::trace!("Incrementing a user score by 1");
    add_score(pool, id, guild_id, 1).await
}

/// Decrement the score of the user identified by the provided
/// id by 1.
pub async fn decrement_score(
    pool: &Pool,
    id: UserId,
    guild_id: GuildId,
) -> Result<(), String> {
    log::trace!("Decrementing a user score by 1");
    add_score(pool, id, guild_id, -1).await
}

/// Add the provided score to the score of the user identified
/// by the provided id. If no such user exists, a new record
/// is added.
async fn add_score(
    pool: &Pool,
    id: UserId,
    guild_id: GuildId,
    score: i8,
) -> Result<(), String> {
    let client = pool.get().await.map_err(|err| err.to_string())?;
    client
        .execute(
            r#"
            INSERT INTO "user"(id, guild_id, score)
            VALUES ($1, $2, $3)
            ON CONFLICT(id, guild_id)
                DO UPDATE
                SET score = "user".score + $3;
            "#,
            &[&(i64::from(id)), &(i64::from(guild_id)), &(score as i64)],
        )
        .await
        .map_err(|err| err.to_string())?;
    Ok(())
}
