-- Add migration script here
CREATE TABLE IF NOT EXISTS mail (
    id BIGINT PRIMARY KEY DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
    sender VARCHAR(255) NOT NULL,
    recipient VARCHAR(255) NOT NULL,
    match_id VARCHAR(255) DEFAULT NULL,
    subject TEXT NOT NULL,
    body TEXT NOT NULL,
    read BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (sender) REFERENCES users(discord_id),
    FOREIGN KEY (recipient) REFERENCES users(discord_id),
    FOREIGN KEY (match_id) REFERENCES matches(match_id)
);
