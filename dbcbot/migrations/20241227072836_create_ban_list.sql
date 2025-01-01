-- Add migration script here
CREATE TABLE IF NOT EXISTS ban_list (
  discord_id_or_player_tag VARCHAR(255) PRIMARY KEY
);
