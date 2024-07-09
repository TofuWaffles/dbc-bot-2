-- Add migration script here
CREATE TABLE IF NOT EXISTS battles (
    id BIGSERIAL PRIMARY KEY,
    battle_time BIGINT NOT NULL,
    event_id BIGINT NOT NULL,
    battle_class_id BIGINT NOT NULL,
    FOREIGN KEY (event_id) REFERENCES events(id),
    FOREIGN KEY (battle_class_id) REFERENCES battle_classes(id)
);
