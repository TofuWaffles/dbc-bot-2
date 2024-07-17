-- Add migration script here
CREATE TABLE IF NOT EXISTS battle_records (
    record_id BIGSERIAL PRIMARY KEY,
    match_id VARCHAR(255),
    FOREIGN KEY (match_id) REFERENCES matches(match_id)
);