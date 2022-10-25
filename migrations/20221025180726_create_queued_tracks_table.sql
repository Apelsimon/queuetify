CREATE TABLE queued_tracks(
    track_uri TEXT NOT NULL,
    PRIMARY KEY (track_uri),
    session_id uuid NOT NULL REFERENCES sessions (id),
    votes INTEGER DEFAULT 0 NOT NULL
);