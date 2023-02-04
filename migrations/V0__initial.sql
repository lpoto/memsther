CREATE TABLE IF NOT EXISTS "user" (
    id bigint NOT NULL,
    guild_id bigint NOT NULL,
    score bigint NOT NULL DEFAULT 0,
    PRIMARY KEY(id, guild_id)
);
