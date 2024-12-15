-- Add migration script here
CREATE TABLE brawl_maps (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL DEFAULT 'Any',
    disabled BOOLEAN NOT NULL DEFAULT FALSE
);

INSERT INTO brawl_maps (id,name,disabled) VALUES (0,'Any',false);