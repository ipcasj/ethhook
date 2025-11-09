#!/bin/bash

# Populate Demo Database with Realistic Blockchain Events
# This creates a fully-featured demo environment with events, deliveries, and statistics

set -e

API_URL="http://104.248.15.178:3000/api/v1"
POSTGRES_EXEC="ssh root@104.248.15.178 docker exec ethhook-postgres psql -U ethhook -d ethhook"

echo "🎬 Populating Demo Database with Blockchain Events"
echo "==================================================="
echo ""

# Get demo user ID and endpoints
echo "📊 Fetching demo account information..."
DEMO_USER_ID=$($POSTGRES_EXEC -t -c "SELECT id FROM users WHERE email='demo@ethhook.com';" | tr -d ' ')
echo "  User ID: $DEMO_USER_ID"

# Get all endpoint IDs
ENDPOINT_IDS=($($POSTGRES_EXEC -t -c "SELECT id FROM endpoints WHERE application_id IN (SELECT id FROM applications WHERE user_id='$DEMO_USER_ID');" | tr -d ' '))
echo "  Found ${#ENDPOINT_IDS[@]} endpoints"
echo ""

# Generate realistic events for past 7 days
echo "📡 Generating blockchain events..."

# Counter for progress
EVENT_COUNT=0

# Function to generate random hex
random_hex() {
  local length=$1
  openssl rand -hex $((length/2)) | tr -d '\n'
}

# Function to generate Ethereum address
random_address() {
  echo "0x$(random_hex 40)"
}

# Function to generate realistic block number (recent Ethereum mainnet)
random_recent_block() {
  # Current Ethereum block ~20,800,000
  echo $((20800000 - RANDOM % 10000))
}

# Function to insert event
insert_event() {
  local endpoint_id=$1
  local contract_addr=$2
  local event_sig=$3
  local hours_ago=$4
  
  local block_num=$(random_recent_block)
  local block_hash="0x$(random_hex 64)"
  local tx_hash="0x$(random_hex 64)"
  local log_index=$((RANDOM % 100))
  local from_addr=$(random_address)
  local to_addr=$(random_address)
  local timestamp=$(date -u -v-${hours_ago}H +"%Y-%m-%d %H:%M:%S")
  
  # Insert event
  local event_id=$($POSTGRES_EXEC -t -c "
    INSERT INTO events (
      block_number, block_hash, transaction_hash, log_index,
      contract_address, topics, data, ingested_at, processed_at
    ) VALUES (
      $block_num,
      '$block_hash',
      '$tx_hash',
      $log_index,
      '$contract_addr',
      ARRAY['$event_sig', '$from_addr', '$to_addr'],
      '0x$(random_hex 128)',
      '$timestamp',
      '$timestamp'
    )
    RETURNING id;
  " | tr -d ' ')
  
  # Insert successful delivery attempt
  local delivery_success=$((RANDOM % 100))
  if [ $delivery_success -lt 95 ]; then
    # 95% success rate
    $POSTGRES_EXEC -c "
      INSERT INTO delivery_attempts (
        event_id, endpoint_id, attempt_number, success,
        http_status_code, response_body, attempted_at, completed_at
      ) VALUES (
        '$event_id',
        '$endpoint_id',
        1,
        true,
        200,
        '{\"status\":\"received\"}',
        '$timestamp',
        '$timestamp'
      );
    " > /dev/null
  else
    # 5% failure rate
    $POSTGRES_EXEC -c "
      INSERT INTO delivery_attempts (
        event_id, endpoint_id, attempt_number, success,
        http_status_code, error_message, attempted_at, completed_at
      ) VALUES (
        '$event_id',
        '$endpoint_id',
        1,
        false,
        500,
        'Connection timeout',
        '$timestamp',
        '$timestamp'
      );
    " > /dev/null
  fi
  
  EVENT_COUNT=$((EVENT_COUNT + 1))
}

# USDT Transfers (very high volume)
echo "  💵 Creating USDT transfer events..."
USDT_ENDPOINT=${ENDPOINT_IDS[11]}  # USDT Transfers endpoint
USDT_CONTRACT="0xdac17f958d2ee523a2206206994597c13d831ec7"
TRANSFER_SIG="0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"

for hour in {1..168}; do  # 7 days
  # 50-100 events per hour for USDT
  for i in $(seq 1 $((50 + RANDOM % 50))); do
    insert_event "$USDT_ENDPOINT" "$USDT_CONTRACT" "$TRANSFER_SIG" "$hour"
  done
  if [ $((hour % 24)) -eq 0 ]; then
    echo "    Day $((hour / 24)): $EVENT_COUNT events"
  fi
done

# USDC Transfers (high volume)
echo "  💰 Creating USDC transfer events..."
USDC_ENDPOINT=${ENDPOINT_IDS[10]}
USDC_CONTRACT="0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"

for hour in {1..168}; do
  for i in $(seq 1 $((30 + RANDOM % 40))); do
    insert_event "$USDC_ENDPOINT" "$USDC_CONTRACT" "$TRANSFER_SIG" "$hour"
  done
done

# Uniswap V3 Swaps (moderate volume)
echo "  🦄 Creating Uniswap swap events..."
UNI_ENDPOINT=${ENDPOINT_IDS[9]}
UNI_CONTRACT="0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640"
SWAP_SIG="0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67"

for hour in {1..168}; do
  for i in $(seq 1 $((20 + RANDOM % 30))); do
    insert_event "$UNI_ENDPOINT" "$UNI_CONTRACT" "$SWAP_SIG" "$hour"
  done
done

# WETH Deposits/Withdrawals (moderate volume)
echo "  💎 Creating WETH deposit/withdrawal events..."
WETH_ENDPOINT=${ENDPOINT_IDS[8]}
WETH_CONTRACT="0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
DEPOSIT_SIG="0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c"

for hour in {1..168}; do
  for i in $(seq 1 $((10 + RANDOM % 20))); do
    insert_event "$WETH_ENDPOINT" "$WETH_CONTRACT" "$DEPOSIT_SIG" "$hour"
  done
done

# Aave V3 (lower volume)
echo "  🏦 Creating Aave V3 events..."
AAVE_ENDPOINT=${ENDPOINT_IDS[7]}
AAVE_CONTRACT="0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2"
SUPPLY_SIG="0x2b627736bca15cd5381dcf80b0bf11fd197d01a037c52b927a881a10fb73ba61"

for hour in {1..168}; do
  for i in $(seq 1 $((5 + RANDOM % 10))); do
    insert_event "$AAVE_ENDPOINT" "$AAVE_CONTRACT" "$SUPPLY_SIG" "$hour"
  done
done

# OpenSea (lower volume, valuable events)
echo "  🖼️  Creating OpenSea NFT sale events..."
OPENSEA_ENDPOINT=${ENDPOINT_IDS[6]}
OPENSEA_CONTRACT="0x00000000000000adc04c56bf30ac9d3c0aaf14dc"
ORDER_SIG="0x9d9af8e38d66c62e2c12f0225249fd9d721c54b83f48d9352c97c6cacdcb6f31"

for hour in {1..168}; do
  for i in $(seq 1 $((2 + RANDOM % 5))); do
    insert_event "$OPENSEA_ENDPOINT" "$OPENSEA_CONTRACT" "$ORDER_SIG" "$hour"
  done
done

# Blur (lower volume)
echo "  ⚡ Creating Blur marketplace events..."
BLUR_ENDPOINT=${ENDPOINT_IDS[5]}
BLUR_CONTRACT="0x000000000000ad05ccc4f10045630fb830b95127"
BLUR_SIG="0x61cbb2a3dee0b6064c2e681aadd61677fb4ef319f0b547508d495626f5a62f64"

for hour in {1..168}; do
  for i in $(seq 1 $((1 + RANDOM % 4))); do
    insert_event "$BLUR_ENDPOINT" "$BLUR_CONTRACT" "$BLUR_SIG" "$hour"
  done
done

# BAYC Transfers (rare but valuable)
echo "  🐵 Creating BAYC NFT transfer events..."
BAYC_ENDPOINT=${ENDPOINT_IDS[4]}
BAYC_CONTRACT="0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d"

for hour in {1..168}; do
  if [ $((RANDOM % 10)) -eq 0 ]; then  # ~10% of hours have activity
    for i in $(seq 1 $((1 + RANDOM % 3))); do
      insert_event "$BAYC_ENDPOINT" "$BAYC_CONTRACT" "$TRANSFER_SIG" "$hour"
    done
  fi
done

# Azuki Transfers
echo "  🌸 Creating Azuki NFT transfer events..."
AZUKI_ENDPOINT=${ENDPOINT_IDS[3]}
AZUKI_CONTRACT="0xed5af388653567af2f388e6224dc7c4b3241c544"

for hour in {1..168}; do
  if [ $((RANDOM % 8)) -eq 0 ]; then
    for i in $(seq 1 $((1 + RANDOM % 2))); do
      insert_event "$AZUKI_ENDPOINT" "$AZUKI_CONTRACT" "$TRANSFER_SIG" "$hour"
    done
  fi
done

# Bridge events (lower volume)
echo "  🌉 Creating bridge deposit events..."
ARB_ENDPOINT=${ENDPOINT_IDS[2]}
ARB_CONTRACT="0x8315177ab297ba92a06054ce80a67ed4dbd7ed3a"
ARB_SIG="0x85291dff2161a93c2f12c819d31889c96c63042116f5bc5a205aa701c2c429f5"

for hour in {1..168}; do
  for i in $(seq 1 $((3 + RANDOM % 7))); do
    insert_event "$ARB_ENDPOINT" "$ARB_CONTRACT" "$ARB_SIG" "$hour"
  done
done

OP_ENDPOINT=${ENDPOINT_IDS[1]}
OP_CONTRACT="0x99c9fc46f92e8a1c0dec1b1747d010903e884be1"
OP_SIG="0x73d170910aba9e6d50b102db522b1dbcd796216f5128b445aa2135272886497e"

for hour in {1..168}; do
  for i in $(seq 1 $((2 + RANDOM % 5))); do
    insert_event "$OP_ENDPOINT" "$OP_CONTRACT" "$OP_SIG" "$hour"
  done
done

BASE_ENDPOINT=${ENDPOINT_IDS[0]}
BASE_CONTRACT="0x49048044d57e1c92a77f79988d21fa8faf74e97e"

for hour in {1..168}; do
  for i in $(seq 1 $((2 + RANDOM % 5))); do
    insert_event "$BASE_ENDPOINT" "$BASE_CONTRACT" "$OP_SIG" "$hour"
  done
done

echo ""
echo "✅ Database population complete!"
echo ""
echo "📊 Final Statistics:"
$POSTGRES_EXEC -c "
SELECT 
  COUNT(*) as total_events,
  COUNT(DISTINCT contract_address) as unique_contracts,
  MIN(ingested_at) as oldest_event,
  MAX(ingested_at) as newest_event
FROM events;
"

echo ""
$POSTGRES_EXEC -c "
SELECT 
  COUNT(*) as total_deliveries,
  COUNT(*) FILTER (WHERE success = true) as successful,
  COUNT(*) FILTER (WHERE success = false) as failed,
  ROUND(100.0 * COUNT(*) FILTER (WHERE success = true) / COUNT(*), 2) as success_rate
FROM delivery_attempts;
"

echo ""
echo "🌐 Demo is ready!"
echo "  URL: http://104.248.15.178:3002"
echo "  Email: demo@ethhook.com"
echo "  Password: Demo1234!"
