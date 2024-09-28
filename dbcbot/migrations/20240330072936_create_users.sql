-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
    discord_id VARCHAR(255) PRIMARY KEY,
    discord_name VARCHAR(255) NOT NULL,
    player_tag VARCHAR(255) NOT NULL,
    player_name VARCHAR(255) NOT NULL,
    icon INT NOT NULL,
    trophies INT NOT NULL,
    brawlers JSONB NOT NULL
    deleted BOOLEAN NOT NULL DEFAULT FALSE
);
