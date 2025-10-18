-- Add session_revoked_at column to user_invitation table
ALTER TABLE user_invitation 
ADD COLUMN session_revoked_at TEXT;

-- Create index for efficient session revoked lookups
CREATE INDEX idx_user_invitation_session_revoked ON user_invitation(session_revoked_at);