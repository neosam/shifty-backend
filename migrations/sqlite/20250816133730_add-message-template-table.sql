-- Add text template table for storing template texts
CREATE TABLE text_template (
    id BLOB PRIMARY KEY NOT NULL,
    template_type TEXT NOT NULL,
    template_text TEXT NOT NULL,
    created_at TEXT,
    created_by TEXT,
    deleted TEXT,
    deleted_by TEXT,
    update_version BLOB NOT NULL,
    update_process TEXT
);

-- Create index on template_type for faster lookups
CREATE INDEX idx_text_template_type ON text_template(template_type);

-- Create index on deleted for soft delete queries
CREATE INDEX idx_text_template_deleted ON text_template(deleted);
