-- Add migration script here
CREATE TABLE IF NOT EXISTS tournament_players (
    discord_id VARCHAR(255) NOT NULL REFERENCES users(discord_id) ON DELETE CASCADE ON UPDATE CASCADE,
    tournament_id INT NOT NULL REFERENCES tournaments(tournament_id) ON DELETE CASCADE ON UPDATE CASCADE,
    PRIMARY KEY (discord_id, tournament_id)
);
