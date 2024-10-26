ALTER TABLE working_hours RENAME TO employee_work_details;

-- Add days where the employee is supposed to work
ALTER TABLE employee_work_details ADD COLUMN monday INTEGER NOT NULL DEFAULT 1;
ALTER TABLE employee_work_details ADD COLUMN tuesday INTEGER NOT NULL DEFAULT 1;
ALTER TABLE employee_work_details ADD COLUMN wednesday INTEGER NOT NULL DEFAULT 1;
ALTER TABLE employee_work_details ADD COLUMN thursday INTEGER NOT NULL DEFAULT 1;
ALTER TABLE employee_work_details ADD COLUMN friday INTEGER NOT NULL DEFAULT 1;
ALTER TABLE employee_work_details ADD COLUMN saturday INTEGER NOT NULL DEFAULT 1;
ALTER TABLE employee_work_details ADD COLUMN sunday INTEGER NOT NULL DEFAULT 0;

-- Start week and end week on the first and last week of the contract
ALTER TABLE employee_work_details ADD COLUMN start_day_of_week INTEGER NOT NULL DEFAULT 1;
ALTER TABLE employee_work_details ADD COLUMN end_day_of_week INTEGER NOT NULL DEFAULT 7;

-- Add vacation days per year
ALTER TABLE employee_work_details ADD COLUMN vacation_days INTEGER NOT NULL DEFAULT 0;

-- Remove days_per_week since they can now be calculated by the weekday columns
ALTER TABLE employee_work_details DROP COLUMN days_per_week;

