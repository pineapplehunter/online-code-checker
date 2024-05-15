-- Add migration script here
ALTER TABLE solutions
ADD COLUMN problem_id TEXT NOT NULL;