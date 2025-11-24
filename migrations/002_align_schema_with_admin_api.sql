-- Migration to align database schema with Admin API code expectations
-- SQLite version

-- Add 'name' column to users (in addition to full_name)
ALTER TABLE users ADD COLUMN name TEXT;
UPDATE users SET name = full_name WHERE name IS NULL;

-- Add api_key to applications table
ALTER TABLE applications ADD COLUMN api_key TEXT;
CREATE UNIQUE INDEX IF NOT EXISTS idx_applications_api_key ON applications(api_key) WHERE api_key IS NOT NULL;

-- SQLite doesn't support RENAME COLUMN directly in older versions
-- Instead we'll add new columns and copy data

-- Add webhook_url (replacing url)
ALTER TABLE endpoints ADD COLUMN webhook_url TEXT;
UPDATE endpoints SET webhook_url = url WHERE webhook_url IS NULL;

-- Add description to endpoints
ALTER TABLE endpoints ADD COLUMN description TEXT;

-- Add chain_ids (stored as JSON text array for SQLite compatibility)
ALTER TABLE endpoints ADD COLUMN chain_ids TEXT;

-- Add contract_addresses (stored as JSON text array)
ALTER TABLE endpoints ADD COLUMN contract_addresses TEXT;
UPDATE endpoints SET contract_addresses = json_array(contract_address) WHERE contract_addresses IS NULL AND contract_address IS NOT NULL;

-- Add event_signatures (replacing event_topics)
ALTER TABLE endpoints ADD COLUMN event_signatures TEXT;
UPDATE endpoints SET event_signatures = event_topics WHERE event_signatures IS NULL;
