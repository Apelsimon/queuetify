CREATE TABLE queued_tracks(
    track_uri TEXT NOT NULL,
    session_id uuid NOT NULL REFERENCES sessions (id),
    PRIMARY KEY (track_uri, session_id),    
    votes INTEGER DEFAULT 0 NOT NULL
);