-- Add up migration script here
CREATE TABLE registrations (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    training_id TEXT NOT NULL,

    FOREIGN KEY (training_id) REFERENCES trainings(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
)
