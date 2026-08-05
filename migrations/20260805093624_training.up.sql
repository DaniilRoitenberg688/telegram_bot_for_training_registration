-- Add up migration script here
CREATE TABLE trainings (
    id TEXT PRIMARY KEY,
    date TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time  TEXT NOT NULL,
    capacity INT,
    enabled BOOLEAN

)
