-- Add user tracking to booking table

ALTER TABLE booking ADD COLUMN created_by TEXT;
ALTER TABLE booking ADD COLUMN deleted_by TEXT;

-- Add foreign key constraints to reference the user table
-- Note: SQLite doesn't support adding foreign key constraints via ALTER TABLE
-- We'll enforce this at the application level for now

-- The created_by should reference user.name
-- The deleted_by should reference user.name 