-- Add up migration script

CREATE TABLE plans(
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    created_by INTEGER NOT NULL,
    created_date TIMESTAMPTZ NOT NULL,

    CONSTRAINT fk_created_by 
        FOREIGN KEY(created_by) 
        REFERENCES users(id)
);
