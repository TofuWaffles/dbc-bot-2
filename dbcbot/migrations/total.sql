-- Add migration script here
CREATE TABLE IF NOT EXISTS config (
    guild_id VARCHAR(255) PRIMARY KEY,
    marshal_role_id VARCHAR(255) NOT NULL,
    announcement_channel_id VARCHAR(255) NOT NULL,
    notification_channel_id VARCHAR(255) NOT NULL,
    log_channel_id VARCHAR(255) NOT NULL
);

-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
    discord_id VARCHAR(255) PRIMARY KEY,
    discord_name VARCHAR(255) NOT NULL,
    player_tag VARCHAR(255) NOT NULL,
    player_name VARCHAR(255) NOT NULL,
    icon INT NOT NULL,
    trophies INT NOT NULL,
    brawlers JSONB NOT NULL
);

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
    map VARCHAR(255),
    wins_required INT NOT NULL
);

-- Add migration script here
CREATE TABLE IF NOT EXISTS tournament_players (
    discord_id VARCHAR(255) NOT NULL REFERENCES users(discord_id) ON DELETE CASCADE ON UPDATE CASCADE,
    tournament_id INT NOT NULL REFERENCES tournaments(tournament_id) ON DELETE CASCADE ON UPDATE CASCADE,
    PRIMARY KEY (discord_id, tournament_id)
);

-- Add migration script here
CREATE TYPE player_type AS ENUM ('player', 'dummy', 'pending');
CREATE TYPE player_number AS ENUM ('player_1', 'player_2');

CREATE TABLE IF NOT EXISTS matches (
    match_id VARCHAR(255) PRIMARY KEY,
    tournament_id INT NOT NULL REFERENCES tournaments(tournament_id) ON DELETE CASCADE ON UPDATE CASCADE,
    round INT NOT NULL,
    sequence_in_round INT NOT NULL,
    player_1_type player_type NOT NULL DEFAULT 'player',
    player_2_type player_type NOT NULL DEFAULT 'player',
    discord_id_1 VARCHAR(255) REFERENCES users(discord_id) ON DELETE SET NULL ON UPDATE CASCADE,
    discord_id_2 VARCHAR(255) REFERENCES users(discord_id) ON DELETE SET NULL ON UPDATE CASCADE,
    player_1_ready BOOLEAN NOT NULL,
    player_2_ready BOOLEAN NOT NULL,
    winner player_number
);

-- Add migration script here
CREATE TABLE IF NOT EXISTS manager_roles (
    guild_id VARCHAR(255) NOT NULL PRIMARY KEY,
    manager_role_id VARCHAR(255) NOT NULL
);

-- Add migration script here

CREATE TABLE IF NOT EXISTS banned_brawlers (
    tournament_id INT NOT NULL REFERENCES tournaments(tournament_id),
    brawler VARCHAR(255) NOT NULL,
    PRIMARY KEY (tournament_id, brawler)
);

-- Add migration script here
CREATE TABLE IF NOT EXISTS battle_records (
    record_id BIGSERIAL PRIMARY KEY,
    match_id VARCHAR(255),
    FOREIGN KEY (match_id) REFERENCES matches(match_id)
);
-- Add migration script here
CREATE TABLE IF NOT EXISTS battles (
    id BIGSERIAL PRIMARY KEY,
    record_id BIGSERIAL NOT NULL,
    battle_time BIGINT NOT NULL,
    FOREIGN KEY (record_id) REFERENCES battle_records(record_id)
);

-- Add migration script here
CREATE TYPE mode AS ENUM (
    'brawl_ball', 'gem_grab', 'heist', 'bounty', 'siege', 'solo_showdown', 
    'duo_showdown', 'hot_zone', 'knockout', 'takedown', 'lone_star', 'big_game', 
    'robo_rumble', 'boss_fight', 'wipe_out', 'duels', 'paint_brawl', 
    'brawl_ball5v5', 'gem_grab5v5', 'bounty5v5', 'knockout5v5'
);
CREATE TYPE battle_type AS ENUM (
    'friendly', 'ranked'
);

CREATE TYPE result AS ENUM (
    'victory', 'defeat', 'draw'
);
CREATE TABLE IF NOT EXISTS battle_classes (
    id BIGSERIAL PRIMARY KEY,
    battle_id BIGINT NOT NULL,
    mode mode NOT NULL,
    battle_type battle_type NOT NULL,
    result result NOT NULL,
    duration INT NOT NULL,
    trophy_change INT,
    teams JSONB NOT NULL,
    FOREIGN KEY (battle_id) REFERENCES battles(id)
);

-- Add migration script here

CREATE TABLE IF NOT EXISTS events (
    id BIGSERIAL PRIMARY KEY,
    mode mode,
    map TEXT,
    battle_id BIGINT,
    FOREIGN KEY (battle_id) REFERENCES battles(id)
);

