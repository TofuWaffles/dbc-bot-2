-- Add migration script here
CREATE TABLE IF NOT EXISTS match_schedules (
    match_id UUID PRIMARY KEY REFERENCES matches(match_id) ON DELETE CASCADE ON UPDATE CASCADE,
    proposed_time INT NOT NULL,
    time_of_proposal TIMESTAMPTZ NOT NULL,
    proposer SMALLINT NOT NULL,
    accepted BOOL NOT NULL
);
