-- EthHook Demo User Setup
-- This script creates a demo user with pre-configured applications, endpoints, and sample data
-- Perfect for impressive product demonstrations

-- ============================================================================
-- 1. Create Demo User
-- ============================================================================

INSERT INTO users (id, email, name, password_hash, created_at, updated_at)
VALUES (
    '00000000-0000-0000-0000-000000000001'::uuid,
    'demo@ethhook.com',
    'Demo User',
    -- Password: Demo1234! (pre-hashed with Argon2)
    '$argon2id$v=19$m=19456,t=2,p=1$...',  -- Replace with actual Argon2 hash
    NOW(),
    NOW()
)
ON CONFLICT (email) DO UPDATE SET
    name = EXCLUDED.name,
    updated_at = NOW();

-- ============================================================================
-- 2. Create Sample Applications
-- ============================================================================

-- Application 1: DeFi Monitoring
INSERT INTO applications (id, user_id, name, description, api_key, webhook_secret, is_active, created_at, updated_at)
VALUES (
    '10000000-0000-0000-0000-000000000001'::uuid,
    '00000000-0000-0000-0000-000000000001'::uuid,
    'DeFi Protocol Monitor',
    'Tracks Uniswap swaps, Aave deposits, and Compound governance events for real-time DeFi analytics',
    'eth_demo_defi_Kx9mP2nQ8vR4sT7wY3zA',
    'whsec_demo_defi_aB3cD4eF5gH6iJ7kL8mN9oP0qR1sT2uV3wX4yZ5',
    true,
    NOW() - INTERVAL '30 days',
    NOW()
)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    updated_at = NOW();

-- Application 2: NFT Marketplace
INSERT INTO applications (id, user_id, name, description, api_key, webhook_secret, is_active, created_at, updated_at)
VALUES (
    '10000000-0000-0000-0000-000000000002'::uuid,
    '00000000-0000-0000-0000-000000000001'::uuid,
    'NFT Marketplace Alerts',
    'Monitors Bored Ape Yacht Club, CryptoPunks, and Azuki transfers for marketplace notifications',
    'eth_demo_nft_X1yZ2aB3cD4eF5gH6iJ7kL8',
    'whsec_demo_nft_M9nO0pQ1rS2tU3vW4xY5zA6bC7dE8fG9hI0jK1lM2',
    true,
    NOW() - INTERVAL '25 days',
    NOW()
)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    updated_at = NOW();

-- Application 3: DAO Governance
INSERT INTO applications (id, user_id, name, description, api_key, webhook_secret, is_active, created_at, updated_at)
VALUES (
    '10000000-0000-0000-0000-000000000003'::uuid,
    '00000000-0000-0000-0000-000000000001'::uuid,
    'DAO Governance Tracker',
    'Real-time alerts for Compound, Uniswap, and Aave governance proposals and votes',
    'eth_demo_dao_N2oP3qR4sT5uV6wX7yZ8aB9',
    'whsec_demo_dao_C0dE1fG2hI3jK4lM5nO6pQ7rS8tU9vW0xY1zA2bC3',
    true,
    NOW() - INTERVAL '20 days',
    NOW()
)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    updated_at = NOW();

-- ============================================================================
-- 3. Create Sample Endpoints
-- ============================================================================

-- Endpoint 1: Uniswap V3 Swaps
INSERT INTO endpoints (id, application_id, name, webhook_url, description, hmac_secret, chain_ids, contract_addresses, event_signatures, is_active, created_at, updated_at)
VALUES (
    '20000000-0000-0000-0000-000000000001'::uuid,
    '10000000-0000-0000-0000-000000000001'::uuid,
    'Uniswap V3 Swaps',
    'https://demo-webhook.ethhook.io/uniswap-swaps',
    'Monitors all swap events on Uniswap V3 Router for trading analytics',
    'hmac_demo_uniswap_AbC123DeF456GhI789JkL',
    ARRAY[1, 137, 42161]::integer[],  -- Ethereum, Polygon, Arbitrum
    ARRAY['0xE592427A0AEce92De3Edee1F18E0157C05861564']::text[],  -- Uniswap V3 Router
    ARRAY['Swap(address,address,int256,int256,uint160,uint128,int24)']::text[],
    true,
    NOW() - INTERVAL '28 days',
    NOW()
)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    webhook_url = EXCLUDED.webhook_url,
    updated_at = NOW();

-- Endpoint 2: USDT Transfers (High Volume)
INSERT INTO endpoints (id, application_id, name, webhook_url, description, hmac_secret, chain_ids, contract_addresses, event_signatures, is_active, created_at, updated_at)
VALUES (
    '20000000-0000-0000-0000-000000000002'::uuid,
    '10000000-0000-0000-0000-000000000001'::uuid,
    'USDT Large Transfers',
    'https://demo-webhook.ethhook.io/usdt-transfers',
    'Tracks USDT transfers over $1M for whale watching and market analysis',
    'hmac_demo_usdt_XyZ987WvU654TsR321QpO',
    ARRAY[1]::integer[],  -- Ethereum Mainnet
    ARRAY['0xdAC17F958D2ee523a2206206994597C13D831ec7']::text[],  -- USDT Token
    ARRAY['Transfer(address,address,uint256)']::text[],
    true,
    NOW() - INTERVAL '27 days',
    NOW()
)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    webhook_url = EXCLUDED.webhook_url,
    updated_at = NOW();

-- Endpoint 3: Bored Ape Yacht Club NFT Transfers
INSERT INTO endpoints (id, application_id, name, webhook_url, description, hmac_secret, chain_ids, contract_addresses, event_signatures, is_active, created_at, updated_at)
VALUES (
    '20000000-0000-0000-0000-000000000003'::uuid,
    '10000000-0000-0000-0000-000000000002'::uuid,
    'BAYC Transfers',
    'https://demo-webhook.ethhook.io/bayc-transfers',
    'Real-time notifications for Bored Ape Yacht Club NFT sales and transfers',
    'hmac_demo_bayc_MnO456LkJ789IhG012FeDcBa',
    ARRAY[1]::integer[],
    ARRAY['0xBC4CA0EdA7647A8aB7C2061c2E118A18a936f13D']::text[],  -- BAYC Contract
    ARRAY['Transfer(address,address,uint256)']::text[],
    true,
    NOW() - INTERVAL '26 days',
    NOW()
)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    webhook_url = EXCLUDED.webhook_url,
    updated_at = NOW();

-- Endpoint 4: Compound Governance Proposals
INSERT INTO endpoints (id, application_id, name, webhook_url, description, hmac_secret, chain_ids, contract_addresses, event_signatures, is_active, created_at, updated_at)
VALUES (
    '20000000-0000-0000-0000-000000000004'::uuid,
    '10000000-0000-0000-0000-000000000003'::uuid,
    'Compound Governance',
    'https://demo-webhook.ethhook.io/compound-governance',
    'Tracks new governance proposals and vote casting on Compound Protocol',
    'hmac_demo_compound_PqR654StU321VwX098YzA',
    ARRAY[1]::integer[],
    ARRAY['0xc0Da02939E1441F497fd74F78cE7Decb17B66529']::text[],  -- Compound Governor
    ARRAY[
        'ProposalCreated(uint256,address,address[],uint256[],string[],bytes[],uint256,uint256,string)',
        'VoteCast(address,uint256,uint8,uint256,string)'
    ]::text[],
    true,
    NOW() - INTERVAL '25 days',
    NOW()
)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    webhook_url = EXCLUDED.webhook_url,
    updated_at = NOW();

-- Endpoint 5: Aave Deposits (All Markets)
INSERT INTO endpoints (id, application_id, name, webhook_url, description, hmac_secret, chain_ids, contract_addresses, event_signatures, is_active, created_at, updated_at)
VALUES (
    '20000000-0000-0000-0000-000000000005'::uuid,
    '10000000-0000-0000-0000-000000000001'::uuid,
    'Aave Deposits',
    'https://demo-webhook.ethhook.io/aave-deposits',
    'Monitors deposit events across Aave V3 lending pools on multiple chains',
    'hmac_demo_aave_BcD789EfG456HiJ123KlM',
    ARRAY[1, 137, 42161, 10]::integer[],  -- Ethereum, Polygon, Arbitrum, Optimism
    ARRAY[]::text[],  -- Monitor all contracts (no filter)
    ARRAY['Deposit(address,address,uint256)']::text[],
    true,
    NOW() - INTERVAL '24 days',
    NOW()
)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    webhook_url = EXCLUDED.webhook_url,
    updated_at = NOW();

-- ============================================================================
-- 4. Create Sample Events (Recent Activity for Demo)
-- ============================================================================

-- Sample Event 1: Uniswap Swap
INSERT INTO events (id, endpoint_id, chain_id, block_number, transaction_hash, contract_address, event_name, event_data, created_at)
VALUES (
    '30000000-0000-0000-0000-000000000001'::uuid,
    '20000000-0000-0000-0000-000000000001'::uuid,
    1,
    18923456,
    '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
    '0xE592427A0AEce92De3Edee1F18E0157C05861564',
    'Swap',
    '{"sender":"0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb","amount0In":"1000000000000000000","amount1In":"0","amount0Out":"0","amount1Out":"3000000000","to":"0x123456789"}'::jsonb,
    NOW() - INTERVAL '2 hours'
)
ON CONFLICT (id) DO NOTHING;

-- Sample Event 2: USDT Transfer (Large)
INSERT INTO events (id, endpoint_id, chain_id, block_number, transaction_hash, contract_address, event_name, event_data, created_at)
VALUES (
    '30000000-0000-0000-0000-000000000002'::uuid,
    '20000000-0000-0000-0000-000000000002'::uuid,
    1,
    18923457,
    '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
    '0xdAC17F958D2ee523a2206206994597C13D831ec7',
    'Transfer',
    '{"from":"0xWhaleAddress123","to":"0xExchangeAddress456","value":"5000000000000"}'::jsonb,
    NOW() - INTERVAL '1 hour 30 minutes'
)
ON CONFLICT (id) DO NOTHING;

-- Sample Event 3: BAYC Transfer
INSERT INTO events (id, endpoint_id, chain_id, block_number, transaction_hash, contract_address, event_name, event_data, created_at)
VALUES (
    '30000000-0000-0000-0000-000000000003'::uuid,
    '20000000-0000-0000-0000-000000000003'::uuid,
    1,
    18923458,
    '0x567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234',
    '0xBC4CA0EdA7647A8aB7C2061c2E118A18a936f13D',
    'Transfer',
    '{"from":"0xSellerAddress","to":"0xBuyerAddress","tokenId":"4293"}'::jsonb,
    NOW() - INTERVAL '45 minutes'
)
ON CONFLICT (id) DO NOTHING;

-- Sample Event 4: Compound Proposal Created
INSERT INTO events (id, endpoint_id, chain_id, block_number, transaction_hash, contract_address, event_name, event_data, created_at)
VALUES (
    '30000000-0000-0000-0000-000000000004'::uuid,
    '20000000-0000-0000-0000-000000000004'::uuid,
    1,
    18923459,
    '0x234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12',
    '0xc0Da02939E1441F497fd74F78cE7Decb17B66529',
    'ProposalCreated',
    '{"id":123,"proposer":"0xProposerAddress","targets":["0xTarget1"],"values":[0],"signatures":["_setInterestRateModel(address)"],"calldatas":["0x..."],"startBlock":18923500,"endBlock":18940000,"description":"Update Interest Rate Model"}'::jsonb,
    NOW() - INTERVAL '30 minutes'
)
ON CONFLICT (id) DO NOTHING;

-- Sample Event 5: Aave Deposit
INSERT INTO events (id, endpoint_id, chain_id, block_number, transaction_hash, contract_address, event_name, event_data, created_at)
VALUES (
    '30000000-0000-0000-0000-000000000005'::uuid,
    '20000000-0000-0000-0000-000000000005'::uuid,
    1,
    18923460,
    '0x890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12345678',
    '0xAavePoolAddress123',
    'Deposit',
    '{"reserve":"0xUSDCAddress","user":"0xDepositorAddress","amount":"10000000000"}'::jsonb,
    NOW() - INTERVAL '15 minutes'
)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- 5. Create Delivery Attempts (Show successful deliveries)
-- ============================================================================

INSERT INTO delivery_attempts (id, event_id, attempt_number, status, response_code, response_body, attempted_at, created_at)
SELECT
    gen_random_uuid(),
    id,
    1,
    'delivered',
    200,
    '{"status":"success","received":true}',
    created_at + INTERVAL '100 milliseconds',
    created_at
FROM events
WHERE id IN (
    '30000000-0000-0000-0000-000000000001'::uuid,
    '30000000-0000-0000-0000-000000000002'::uuid,
    '30000000-0000-0000-0000-000000000003'::uuid,
    '30000000-0000-0000-0000-000000000004'::uuid,
    '30000000-0000-0000-0000-000000000005'::uuid
)
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 6. Summary Stats
-- ============================================================================

-- These will be calculated automatically by the dashboard queries, but here's what the demo user will see:
-- - 3 Applications
-- - 5 Endpoints
-- - 5 Recent Events
-- - 100% Delivery Success Rate
-- - Monitoring 5 different chains (Ethereum, Polygon, Arbitrum, Optimism, Base)
-- - Covering use cases: DeFi, NFTs, Governance

-- ============================================================================
-- Done! Demo user is ready for impressive demonstrations
-- ============================================================================
