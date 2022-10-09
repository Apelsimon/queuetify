CREATE TABLE sessions(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    token TEXT NOT NULL,
    queue_id uuid NOT NULL,
    created_at timestamptz NOT NULL
);