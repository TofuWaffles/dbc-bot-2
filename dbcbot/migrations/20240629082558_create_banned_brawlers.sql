-- Add migration script here

CREATE TABLE IF NOT EXISTS banned_brawlers (
    tournament_id INT NOT NULL REFERENCES tournaments(tournament_id),
    brawler VARCHAR(255) NOT NULL,
    PRIMARY KEY (tournament_id, brawler)
);
