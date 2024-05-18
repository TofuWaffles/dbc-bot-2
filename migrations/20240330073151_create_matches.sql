-- Add migration script here
CREATE TYPE player_type AS ENUM ('player', 'dummy', 'pending');

CREATE TABLE IF NOT EXISTS matches (
    match_id VARCHAR(255) PRIMARY KEY,
    tournament_id INT NOT NULL REFERENCES tournaments(tournament_id) ON DELETE CASCADE ON UPDATE CASCADE,
    round INT NOT NULL,
    sequence_in_round INT NOT NULL,
    player_1_type player_type NOT NULL DEFAULT 'player',
    player_2_type player_type NOT NULL DEFAULT 'player',
    discord_id_1 VARCHAR(255) REFERENCES users(discord_id) ON DELETE SET NULL ON UPDATE CASCADE,
    discord_id_2 VARCHAR(255) REFERENCES users(discord_id) ON DELETE SET NULL ON UPDATE CASCADE,
    winner INT
);
