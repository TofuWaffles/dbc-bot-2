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
    mode mode NOT NULL,
    battle_type battle_type NOT NULL,
    result result NOT NULL,
    duration BIGINT NOT NULL,
    trophy_change BIGINT,
    teams JSONB NOT NULL
);
