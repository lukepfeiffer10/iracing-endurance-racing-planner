-- Add up migration script here

CREATE TABLE user_plans(
    user_id INTEGER,
    plan_id UUID,

    PRIMARY KEY(user_id, plan_id),
    CONSTRAINT fk_plan_id
        FOREIGN KEY(plan_id)
        REFERENCES plans(id),
    CONSTRAINT fk_user_id
        FOREIGN KEY(user_id)
        REFERENCES users(id)
);
