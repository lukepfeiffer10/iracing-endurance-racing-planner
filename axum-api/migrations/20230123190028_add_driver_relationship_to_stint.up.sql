-- Add up migration script here

ALTER TABLE stints
    ADD driver_id INTEGER;

ALTER TABLE stints
    ADD CONSTRAINT fk_driver_id
        FOREIGN KEY(driver_id)
        REFERENCES drivers(id);
