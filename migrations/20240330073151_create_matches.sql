-- Add migration script here
CREATE TABLE IF NOT EXISTS matches (
    match_id UUID PRIMARY KEY,
    tournament_id INT NOT NULL REFERENCES tournaments(tournament_id) ON DELETE CASCADE ON UPDATE CASCADE,
    round INT NOT NULL,
    sequence_in_round INT NOT NULL,
    player1_id VARCHAR(255) NOT NULL REFERENCES players(discord_id) ON DELETE SET NULL ON UPDATE CASCADE,
    player2_id VARCHAR(255) NOT NULL REFERENCES players(discord_id) ON DELETE SET NULL ON UPDATE CASCADE,
    winner INT
);
