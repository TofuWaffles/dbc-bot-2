-- Add migration script here
CREATE TABLE IF NOT EXISTS match_schedules (
    match_id UUID PRIMARY KEY REFERENCES matches(match_id) ON DELETE CASCADE ON UPDATE CASCADE,
    proposed_time INT,
    proposer SMALLINT,
    accepted BOOL
);
