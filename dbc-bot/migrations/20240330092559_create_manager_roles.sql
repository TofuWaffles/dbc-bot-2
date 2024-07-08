-- Add migration script here
CREATE TABLE IF NOT EXISTS manager_roles (
    guild_id VARCHAR(255) NOT NULL PRIMARY KEY,
    manager_role_id VARCHAR(255) NOT NULL
);
