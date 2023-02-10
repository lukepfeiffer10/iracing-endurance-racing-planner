-- Add up migration script here

CREATE TABLE event_configs(
    plan_id UUID NOT NULL PRIMARY KEY,

    race_duration INTERVAL NOT NULL,
    session_start_utc TIMESTAMPTZ NOT NULL,
    race_start_tod TIMESTAMP NOT NULL,
    green_flag_offset INTERVAL NOT NULL,

    CONSTRAINT fk_plan_id
        FOREIGN KEY(plan_id)
        REFERENCES plans(id)
);
