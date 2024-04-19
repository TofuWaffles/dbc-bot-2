-- Add migration script here
CREATE TABLE IF NOT EXISTS tournaments (
    tournament_id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    guild_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    start_time TIMESTAMPTZ,
    active BOOL NOT NULL,
    started BOOL NOT NULL
);
