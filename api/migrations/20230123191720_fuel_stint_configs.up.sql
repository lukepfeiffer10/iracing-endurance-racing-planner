-- Add up migration script here

CREATE TABLE fuel_stint_configs(
    plan_id UUID PRIMARY KEY,

    pit_duration INTERVAL NOT NULL,
    fuel_tank_size INTEGER NOT NULL,
    tire_change_time INTERVAL NOT NULL,
    add_tire_time BOOLEAN NOT NULL,

    CONSTRAINT fk_plan_id
        FOREIGN KEY(plan_id)
        REFERENCES plans(id)        
);
