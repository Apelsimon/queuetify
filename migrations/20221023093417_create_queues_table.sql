CREATE TYPE queue_entry AS (
    track_uri TEXT,
    votes INTEGER
);

-- TODO should sessions reference queues id instead?
CREATE TABLE queues(
    id uuid NOT NULL REFERENCES sessions (queue_id),
    PRIMARY KEY (id),
    current_track_uri TEXT,
    tracks queue_entry[]
);