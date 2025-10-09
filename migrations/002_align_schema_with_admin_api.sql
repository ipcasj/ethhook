-- Migration to align database schema with Admin API code expectations
-- This adds columns and makes adjustments to match the handlers

-- Add 'name' column to users (in addition to full_name)
-- We'll keep both: full_name for display, name as an alias
ALTER TABLE users ADD COLUMN IF NOT EXISTS name VARCHAR(255);
UPDATE users SET name = full_name WHERE name IS NULL;

-- Add api_key to applications table
ALTER TABLE applications ADD COLUMN IF NOT EXISTS api_key VARCHAR(255);
CREATE UNIQUE INDEX IF NOT EXISTS idx_applications_api_key ON applications(api_key) WHERE api_key IS NOT NULL;

-- Rename url to webhook_url in endpoints
ALTER TABLE endpoints RENAME COLUMN url TO webhook_url;

-- Add description to endpoints (separate from name)
ALTER TABLE endpoints ADD COLUMN IF NOT EXISTS description TEXT;

-- Add chain_ids array to endpoints (multi-chain support)
ALTER TABLE endpoints ADD COLUMN IF NOT EXISTS chain_ids INTEGER[];

-- Rename contract_address to contract_addresses (array for multiple contracts)
ALTER TABLE endpoints RENAME COLUMN contract_address TO contract_addresses;
ALTER TABLE endpoints ALTER COLUMN contract_addresses TYPE TEXT[] USING ARRAY[contract_addresses];

-- Rename event_topics to event_signatures (more semantic)
ALTER TABLE endpoints RENAME COLUMN event_topics TO event_signatures;
