-- Add name field to text_template table to make templates more user-friendly
ALTER TABLE text_template ADD COLUMN name TEXT;

-- Create index on name for faster searches
CREATE INDEX idx_text_template_name ON text_template(name);