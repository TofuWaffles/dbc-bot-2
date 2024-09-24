-- Add migration script here
CREATE TYPE mail_type AS ENUM ('user', 'marshal');

CREATE TABLE IF NOT EXISTS mail (
    id BIGINT PRIMARY KEY DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
    sender VARCHAR(255) NOT NULL,
    recipient VARCHAR(255) NOT NULL,
    match_id VARCHAR(255) NOT NULL DEFAULT '',
    subject TEXT NOT NULL,
    body TEXT NOT NULL,
    read BOOLEAN NOT NULL DEFAULT FALSE,
    mode mail_type NOT NULL DEFAULT 'user',
    FOREIGN KEY (sender) REFERENCES users(discord_id),
    FOREIGN KEY (recipient) REFERENCES users(discord_id)
);
