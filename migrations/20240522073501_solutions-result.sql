-- Add migration script here
ALTER TABLE solutions
ADD COLUMN stdout TEXT;
ALTER TABLE solutions
ADD COLUMN stderr TEXT;