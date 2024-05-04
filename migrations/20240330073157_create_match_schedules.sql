-- Add migration script here
CREATE TABLE IF NOT EXISTS match_schedules (
    match_id UUID PRIMARY KEY REFERENCES matches(match_id) ON DELETE CASCADE ON UPDATE CASCADE,
    proposed_time BIGINT NOT NULL,
    time_of_proposal BIGINT NOT NULL,
    proposer SMALLINT NOT NULL,
    accepted BOOL NOT NULL
);
