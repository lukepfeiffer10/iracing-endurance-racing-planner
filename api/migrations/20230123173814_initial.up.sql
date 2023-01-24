-- Add up migration script here

CREATE TABLE users(
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    oauth_id TEXT NOT NULL UNIQUE
);
