CREATE TABLE votes(
    client_id uuid NOT NULL,
    session_id uuid NOT NULL REFERENCES sessions (id),
    track_uri TEXT NOT NULL,
    PRIMARY KEY (client_id, track_uri)    
)