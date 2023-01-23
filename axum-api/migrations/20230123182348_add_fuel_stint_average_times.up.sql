-- Add up migration script here

CREATE TABLE fuel_stint_average_times(
    plan_id UUID NOT NULL,

    lap_time INTERVAL NOT NULL,
    fuel_per_lap REAL NOT NULL,
    lap_count INTEGER NOT NULL,
    lap_time_with_pit INTERVAL NOT NULL,
    track_time INTERVAL NOT NULL,
    track_time_with_pit INTERVAL NOT NULL,
    fuel_per_stint REAL NOT NULL,
    stint_type SMALLINT NOT NULL,

    PRIMARY KEY(plan_id, stint_type),
    CONSTRAINT fk_plan_id
        FOREIGN KEY(plan_id)
        REFERENCES plans(id)
);
