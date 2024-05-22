-- Add migration script here
ALTER TABLE solutions
ADD COLUMN created_at datetime;
UPDATE solutions SET created_at = CURRENT_TIMESTAMP;
ALTER TABLE solutions
ADD COLUMN executed_at datetime;

ALTER TABLE user
ADD COLUMN created_at datetime;
UPDATE user SET created_at = CURRENT_TIMESTAMP;
