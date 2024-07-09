-- Add migration script here
CREATE TABLE IF NOT EXISTS battle_records (
    record_id BIGSERIAL PRIMARY KEY,
    match_id TEXT,
    battle_id JSONB
);