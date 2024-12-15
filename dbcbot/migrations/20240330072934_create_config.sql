-- Add migration script here
CREATE TABLE IF NOT EXISTS config (
    guild_id VARCHAR(255) PRIMARY KEY,
    marshal_role_id VARCHAR(255) NOT NULL,
    log_channel_id VARCHAR(255) NOT NULL,
    announcement_channel_id VARCHAR(255) NOT NULL
);
