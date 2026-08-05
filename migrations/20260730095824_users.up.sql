-- Add up migration script here
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    full_name TEXT NOT NULL,
    username TEXT,
    first_name TEXT,
    last_name TEXT,
    is_trainer BOOLEAN
)

