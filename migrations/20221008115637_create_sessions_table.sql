CREATE TABLE sessions(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    token TEXT NOT NULL,
    current_track_uri TEXT,
    created_at timestamptz NOT NULL
);