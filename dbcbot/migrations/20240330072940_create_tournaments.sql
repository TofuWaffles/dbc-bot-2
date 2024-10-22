-- Add migration script here
CREATE TYPE tournament_status AS ENUM ('pending', 'started', 'paused', 'inactive');
CREATE TYPE mode AS ENUM (
    'brawl_ball', 'gem_grab', 'heist', 'bounty', 'siege', 'solo_showdown', 
    'duo_showdown', 'hot_zone', 'knockout', 'takedown', 'lone_star', 'big_game', 
    'robo_rumble', 'boss_fight', 'wipeout', 'duels', 'paint_brawl', 
    'brawl_ball5v5', 'gem_grab5v5', 'bounty5v5', 'knockout5v5', 'unknown'
);
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
    mode mode NOT NULL DEFAULT 'unknown',
    map INT NOT NULL DEFAULT 0,
    wins_required INT NOT NULL,
    announcement_channel_id VARCHAR(255) NOT NULL,
    notification_channel_id VARCHAR(255) NOT NULL
);
