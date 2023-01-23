-- Add up migration script here

ALTER TABLE event_configs
    ADD race_start_utc TIMESTAMPTZ NOT NULL DEFAULT now();

ALTER TABLE event_configs
    ADD race_end_utc TIMESTAMPTZ NOT NULL DEFAULT now();

ALTER TABLE event_configs
    ADD race_end_tod TIMESTAMP NOT NULL DEFAULT now();

ALTER TABLE event_configs
    ADD tod_offset INTERVAL NOT NULL DEFAULT 'PT0S';

ALTER TABLE event_configs
    ALTER COLUMN race_start_utc DROP DEFAULT;

ALTER TABLE event_configs
    ALTER COLUMN race_end_utc DROP DEFAULT;

ALTER TABLE event_configs
    ALTER COLUMN race_end_tod DROP DEFAULT;

ALTER TABLE event_configs
    ALTER COLUMN tod_offset DROP DEFAULT;
