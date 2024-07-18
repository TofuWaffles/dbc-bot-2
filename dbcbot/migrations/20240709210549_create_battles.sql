-- Add migration script here
CREATE TABLE IF NOT EXISTS battles (
    id BIGSERIAL PRIMARY KEY,
    record_id BIGSERIAL NOT NULL,
    battle_time BIGINT NOT NULL,
    FOREIGN KEY (record_id) REFERENCES battle_records(record_id)
);
