-- Add migration script here

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
