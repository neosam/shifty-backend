-- Add session tracking columns to user_invitation table
ALTER TABLE user_invitation 
ADD COLUMN redeemed_at TEXT;

ALTER TABLE user_invitation 
ADD COLUMN session_id TEXT REFERENCES session(id) ON DELETE SET NULL;

-- Create index for efficient session lookups
CREATE INDEX idx_user_invitation_session ON user_invitation(session_id);

-- Create index for finding unredeemed tokens
CREATE INDEX idx_user_invitation_redeemed ON user_invitation(redeemed_at);