-- Add is_admin field to users table for admin-only features
-- SQLite version
ALTER TABLE users ADD COLUMN is_admin INTEGER DEFAULT 0;

-- Create index for admin queries
CREATE INDEX IF NOT EXISTS idx_users_is_admin ON users(is_admin) WHERE is_admin = 1;

-- Set your admin user (update with your actual email)
-- IMPORTANT: Run this manually with your admin email after migration
-- UPDATE users SET is_admin = 1 WHERE email = 'your-admin-email@example.com';
