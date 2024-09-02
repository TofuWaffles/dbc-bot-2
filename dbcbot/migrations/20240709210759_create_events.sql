-- Add migration script here

CREATE TABLE IF NOT EXISTS events (
    id BIGSERIAL PRIMARY KEY,
    mode mode,
    map INT,
    battle_id BIGINT,
    FOREIGN KEY (battle_id) REFERENCES battles(id)
);
