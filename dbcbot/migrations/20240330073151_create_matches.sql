-- Add migration script here
CREATE TABLE IF NOT EXISTS matches (
    match_id VARCHAR(255) PRIMARY KEY,
    tournament_id INT NOT NULL REFERENCES tournaments(tournament_id) ON DELETE CASCADE ON UPDATE CASCADE,
    round INT NOT NULL,
    sequence_in_round INT NOT NULL,
    winner VARCHAR(255),
    score VARCHAR(255) NOT NULL
);
