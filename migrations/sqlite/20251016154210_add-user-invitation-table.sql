-- Add migration script here
CREATE TABLE user_invitation (
    id TEXT NOT NULL PRIMARY KEY,
    username TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    expiration_date TEXT NOT NULL,
    created_date TEXT NOT NULL DEFAULT (datetime('now')),
    update_process TEXT NOT NULL,
    CONSTRAINT fk_user FOREIGN KEY (username) REFERENCES user(name) ON DELETE CASCADE
);

-- Create index on token for fast lookups
CREATE INDEX idx_user_invitation_token ON user_invitation(token);

-- Create index on expiration_date for cleanup queries
CREATE INDEX idx_user_invitation_expiration ON user_invitation(expiration_date);