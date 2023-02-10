-- Add up migration script here

CREATE TABLE stints(
    id UUID NOT NULL PRIMARY KEY,
    plan_id UUID NOT NULL,

    stint_type SMALLINT NOT NULL,
    number INTEGER NOT NULL,
    utc_start TIMESTAMPTZ NOT NULL,
    utc_end TIMESTAMPTZ NOT NULL,
    tod_start TIMESTAMP NOT NULL,
    tod_end TIMESTAMP NOT NULL,
    actual_end TIMESTAMPTZ NOT NULL,
    duration_delta INTERVAL NOT NULL,
    damage_modifier INTERVAL NOT NULL,
    calculated_laps INTEGER NOT NULL,
    actual_laps INTEGER NOT NULL,
    driver_stint_count INTEGER NOT NULL,

    CONSTRAINT fk_plan_id
        FOREIGN KEY(plan_id)
        REFERENCES plans(id)
);
