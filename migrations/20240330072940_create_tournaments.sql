-- Add migration script here
CREATE TYPE tournament_status AS ENUM ('pending', 'started', 'paused', 'inactive');

CREATE TABLE IF NOT EXISTS tournaments (
    tournament_id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    guild_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    start_time TIMESTAMPTZ,
    status tournament_status NOT NULL DEFAULT 'pending'
);
