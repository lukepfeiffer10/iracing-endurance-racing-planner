-- Add up migration script here

CREATE TABLE drivers(
    id SERIAL PRIMARY KEY,
    plan_id UUID NOT NULL,
    
    name TEXT NOT NULL,
    color TEXT NOT NULL,
    utc_offset SMALLINT NOT NULL DEFAULT 0,
    irating SMALLINT NOT NULL DEFAULT 0,
    stint_preference SMALLINT NOT NULL DEFAULT 0,

    CONSTRAINT fk_plan_id
        FOREIGN KEY(plan_id)
        REFERENCES plans(id)
);
