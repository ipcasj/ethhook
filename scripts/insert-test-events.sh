#!/bin/bash
# Script to insert realistic test events into ClickHouse for demo purposes

set -e

echo "Inserting test events into ClickHouse..."

# Get endpoint IDs from database
ssh root@104.248.15.178 'docker exec -i ethhook-clickhouse clickhouse-client --query "
INSERT INTO ethhook.events 
SELECT 
    generateUUIDv4() as id,
    toUUID('\''41721389-257f-4492-1188-029e39eb888b'\'') as endpoint_id,
    toUUID('\''aaf09b06-9349-3a3b-f4fc-9159cb11661b'\'') as application_id,
    toUUID('\''8f1a83f1-598f-6153-0787-2e5ab6e87771'\'') as user_id,
    1 as chain_id,
    21400000 + number as block_number,
    concat('\''0x'\'', substring(MD5(toString(number)), 1, 64)) as block_hash,
    concat('\''0x'\'', substring(MD5(toString(number * 2)), 1, 64)) as transaction_hash,
    number % 10 as log_index,
    '\''0xdac17f958d2ee523a2206206994597c13d831ec7'\'' as contract_address,
    ['\''0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef'\''] as topics,
    '\''0x0000000000000000000000000000000000000000000000000000000005f5e100'\'' as data,
    now() - (100 - number) * 60 as ingested_at,
    now() - (100 - number) * 60 as processed_at
FROM numbers(50)
"
'

echo "✓ Inserted 50 USDT Transfer events"

ssh root@104.248.15.178 'docker exec -i ethhook-clickhouse clickhouse-client --query "
INSERT INTO ethhook.events 
SELECT 
    generateUUIDv4() as id,
    toUUID('\''41721389-257f-4492-1188-029e39eb888c'\'') as endpoint_id,
    toUUID('\''aaf09b06-9349-3a3b-f4fc-9159cb11661b'\'') as application_id,
    toUUID('\''8f1a83f1-598f-6153-0787-2e5ab6e87771'\'') as user_id,
    1 as chain_id,
    21400000 + number as block_number,
    concat('\''0x'\'', substring(MD5(toString(number + 1000)), 1, 64)) as block_hash,
    concat('\''0x'\'', substring(MD5(toString(number * 3)), 1, 64)) as transaction_hash,
    number % 10 as log_index,
    '\''0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48'\'' as contract_address,
    ['\''0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef'\''] as topics,
    '\''0x00000000000000000000000000000000000000000000000000000000000186a0'\'' as data,
    now() - (100 - number) * 60 as ingested_at,
    now() - (100 - number) * 60 as processed_at
FROM numbers(50)
"
'

echo "✓ Inserted 50 USDC Transfer events"

# Verify total count
TOTAL=$(ssh root@104.248.15.178 'docker exec ethhook-clickhouse clickhouse-client --query "SELECT COUNT(*) FROM ethhook.events"')

echo "✓ Total events in database: $TOTAL"
echo ""
echo "You can now refresh the UI to see the test data!"
echo "Dashboard: http://104.248.15.178:3000/dashboard"
echo "Events: http://104.248.15.178:3000/dashboard/events"
