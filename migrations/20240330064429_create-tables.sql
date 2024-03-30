-- Add migration script here
CREATE TABLE IF NOT EXISTS config (
    guild_id VARCHAR(255) PRIMARY KEY,
    manager_role_id VARCHAR(255) NOT NULL,
    host_role_id VARCHAR(255) NOT NULL,
    announcement_channel_id VARCHAR(255) NOT NULL,
    notification_channel_id VARCHAR(255) NOT NULL,
    log_channel_id VARCHAR(255) NOT NULL
);

CREATE TABLE IF NOT EXISTS tournaments (
    tournament_id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    guild_id VARCHAR(255) NOT NULL,
    start_time TIMESTAMP,
    active BOOL NOT NULL
);

CREATE TABLE IF NOT EXISTS players (
    discord_id VARCHAR(255) PRIMARY KEY,
    player_tag VARCHAR(255) NOT NULL
);

CREATE TABLE IF NOT EXISTS tournament_players (
    discord_id VARCHAR(255) NOT NULL REFERENCES players(discord_id) ON DELETE CASCADE ON UPDATE CASCADE,
    tournament_id INT NOT NULL REFERENCES tournaments(tournament_id) ON DELETE CASCADE ON UPDATE CASCADE,
    PRIMARY KEY (discord_id, tournament_id)
);

CREATE TABLE IF NOT EXISTS matches (
    match_id UUID PRIMARY KEY,
    tournament_id INT NOT NULL REFERENCES tournaments(tournament_id) ON DELETE CASCADE ON UPDATE CASCADE,
    round INT NOT NULL,
    round_sequence INT NOT NULL,
    player1_id VARCHAR(255) NOT NULL REFERENCES players(discord_id) ON DELETE SET NULL ON UPDATE CASCADE,
    player2_id VARCHAR(255) NOT NULL REFERENCES players(discord_id) ON DELETE SET NULL ON UPDATE CASCADE,
    winner INT
);

CREATE TABLE IF NOT EXISTS match_schedules (
    match_id UUID PRIMARY KEY REFERENCES matches(match_id) ON DELETE CASCADE ON UPDATE CASCADE,
    proposed_time INT,
    proposer SMALLINT,
    accepted BOOL
);
