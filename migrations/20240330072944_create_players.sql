-- Add migration script here
CREATE TABLE IF NOT EXISTS players (
    discord_id VARCHAR(255) PRIMARY KEY,
    player_tag VARCHAR(255) NOT NULL
);
