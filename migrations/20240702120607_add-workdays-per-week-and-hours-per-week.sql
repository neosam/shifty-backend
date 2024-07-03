-- Add migration script here

ALTER TABLE working_hours ADD COLUMN workdays_per_week INT NOT NULL DEFAULT 5;
ALTER TABLE working_hours ADD COLUMN days_per_week INT NOT NULL DEFAULT 6;