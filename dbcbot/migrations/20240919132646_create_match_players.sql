-- Add migration script here
CREATE TYPE player_type AS ENUM ('player', 'dummy', 'pending');

CREATE TABLE IF NOT EXISTS match_players (
    match_id VARCHAR(255) PRIMARY KEY,
    discord_id VARCHAR(255) REFERENCES users(discord_id) NOT NULL,
    player_type player_type NOT NULL,
    ready BOOLEAN NOT NULL
);
