-- Add down migration script here

ALTER TABLE event_configs
    DROP COLUMN race_start_utc;

ALTER TABLE event_configs
    DROP COLUMN race_end_utc;

ALTER TABLE event_configs
    DROP COLUMN race_end_tod;

ALTER TABLE event_configs
    DROP COLUMN tod_offset;
