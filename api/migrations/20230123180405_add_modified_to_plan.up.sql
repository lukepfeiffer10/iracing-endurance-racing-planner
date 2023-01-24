-- Add up migration script here

ALTER TABLE plans 
    ADD modified_by INTEGER;

ALTER TABLE plans
    ADD modified_date TIMESTAMPTZ;

ALTER TABLE plans
    ADD CONSTRAINT fk_modified_by
            FOREIGN KEY(modified_by) 
            REFERENCES users(id)

