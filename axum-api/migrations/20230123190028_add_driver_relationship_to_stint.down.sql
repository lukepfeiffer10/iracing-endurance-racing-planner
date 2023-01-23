-- Add down migration script here

ALTER TABLE
    DROP CONSTRAINT fk_driver_id;

ALTER TABLE
    DROP COLUMN driver_id;
