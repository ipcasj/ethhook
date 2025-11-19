-- Add is_admin field to users table for admin-only features
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_admin BOOLEAN DEFAULT false;

-- Create index for admin queries
CREATE INDEX IF NOT EXISTS idx_users_is_admin ON users(is_admin) WHERE is_admin = true;

-- Set your admin user (update with your actual email)
-- IMPORTANT: Run this manually with your admin email after migration
-- UPDATE users SET is_admin = true WHERE email = 'your-admin-email@example.com';
