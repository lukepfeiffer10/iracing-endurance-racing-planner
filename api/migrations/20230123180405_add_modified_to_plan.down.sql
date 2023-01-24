-- Add down migration script here

ALTER TABLE plans
    DROP CONSTRAINT fk_modified_by;

ALTER TABLE plans
    DROP COLUMN modified_date;

ALTER TABLE plans
    DROP COLUMN modified_by;
