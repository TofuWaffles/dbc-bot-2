-- Add migration script here
CREATE TABLE brawl_maps (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL DEFAULT 'Any'
);