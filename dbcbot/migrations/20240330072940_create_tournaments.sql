-- Add migration script here
CREATE TYPE tournament_status AS ENUM ('pending', 'started', 'paused', 'inactive');

CREATE TABLE IF NOT EXISTS tournaments (
    tournament_id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    guild_id VARCHAR(255) NOT NULL,
    rounds INT NOT NULL,
    current_round INT NOT NULL,
    created_at BIGINT NOT NULL,
    start_time BIGINT,
    tournament_role_id VARCHAR(255) NOT NULL,
    status tournament_status NOT NULL DEFAULT 'pending',
    map VARCHAR(255)
);
