-- Add migration script here
CREATE TABLE IF NOT EXISTS matches (
    match_id VARCHAR(255) PRIMARY KEY,
    winner VARCHAR(255) DEFAULT NULL,
    score VARCHAR(255) NOT NULL
    start BIGINT DEFAULT NULL,
    "end" BIGINT DEFAULT NULL,
);
