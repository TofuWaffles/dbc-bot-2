-- Add migration script here

CREATE TABLE IF NOT EXISTS events (
    id BIGSERIAL PRIMARY KEY,
    mode mode,
    map TEXT
);
