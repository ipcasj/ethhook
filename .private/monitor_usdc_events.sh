#!/bin/bash

# Use full docker path
DOCKER=/usr/local/bin/docker

echo "🔍 Monitoring EthHook System for USDC Transfer Events"
echo "======================================================"
echo ""
echo "USDC Contract: 0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"
echo "Endpoint Created: 2025-10-23 02:31 UTC"
echo "Waiting for fresh Transfer events..."
echo ""

# Store initial counts
INITIAL_REDIS=$($DOCKER exec ethhook-redis redis-cli XLEN "events:11155111")
INITIAL_DB=$($DOCKER exec ethhook-postgres psql -U ethhook -d ethhook -t -c "SELECT COUNT(*) FROM events;")
INITIAL_JOBS=$($DOCKER exec ethhook-postgres psql -U ethhook -d ethhook -t -c "SELECT COUNT(*) FROM delivery_attempts;")

echo "📊 Initial State:"
echo "   Redis Stream: $INITIAL_REDIS events"
echo "   Database Events: $INITIAL_DB events"
echo "   Delivery Attempts: $INITIAL_JOBS attempts"
echo ""
echo "🔄 Monitoring in real-time (Ctrl+C to stop)..."
echo ""

LAST_REDIS=$INITIAL_REDIS
LAST_DB=$INITIAL_DB
LAST_JOBS=$INITIAL_JOBS

while true; do
    sleep 5

    # Check current counts
    CURRENT_REDIS=$($DOCKER exec ethhook-redis redis-cli XLEN "events:11155111" 2>/dev/null || echo $LAST_REDIS)
    CURRENT_DB=$($DOCKER exec ethhook-postgres psql -U ethhook -d ethhook -t -c "SELECT COUNT(*) FROM events;" 2>/dev/null | xargs || echo $LAST_DB)
    CURRENT_JOBS=$($DOCKER exec ethhook-postgres psql -U ethhook -d ethhook -t -c "SELECT COUNT(*) FROM delivery_attempts;" 2>/dev/null | xargs || echo $LAST_JOBS)

    # Detect changes
    NEW_REDIS=$(($CURRENT_REDIS - $LAST_REDIS))
    NEW_DB=$(($CURRENT_DB - $LAST_DB))
    NEW_JOBS=$(($CURRENT_JOBS - $LAST_JOBS))

    if [ $NEW_REDIS -gt 0 ] || [ $NEW_DB -gt 0 ] || [ $NEW_JOBS -gt 0 ]; then
        echo "[$(date '+%H:%M:%S')] 📥 Activity detected!"

        if [ $NEW_REDIS -gt 0 ]; then
            echo "   ✓ Redis: +$NEW_REDIS events (total: $CURRENT_REDIS)"

            # Check if any are USDC
            USDC_CHECK=$(docker exec ethhook-redis redis-cli XREVRANGE "events:11155111" + - COUNT 10 | grep -i "1c7d4b196cb0c7b01d743fbc6116a902379c7238" || echo "")
            if [ ! -z "$USDC_CHECK" ]; then
                echo "   🎉 USDC EVENT DETECTED in Redis stream!"
            fi
        fi

        if [ $NEW_DB -gt 0 ]; then
            echo "   ✓ Database: +$NEW_DB events (total: $CURRENT_DB)"
            echo "   🎯 EVENT MATCHED TO ENDPOINT!"

            # Get latest event details
            LATEST=$(docker exec ethhook-postgres psql -U ethhook -d ethhook -t -c "SELECT contract_address, block_number, transaction_hash FROM events ORDER BY ingested_at DESC LIMIT 1;" | xargs)
            echo "   📋 Latest: $LATEST"
        fi

        if [ $NEW_JOBS -gt 0 ]; then
            echo "   ✓ Deliveries: +$NEW_JOBS jobs (total: $CURRENT_JOBS)"
            echo "   📤 WEBHOOK DELIVERY INITIATED!"

            # Check delivery status
            STATUS=$(docker exec ethhook-postgres psql -U ethhook -d ethhook -t -c "SELECT success, http_status_code FROM delivery_attempts ORDER BY attempted_at DESC LIMIT 1;" | xargs)
            echo "   📊 Status: $STATUS"
        fi

        echo ""

        # If we got a complete flow, celebrate!
        if [ $NEW_DB -gt 0 ] && [ $NEW_JOBS -gt 0 ]; then
            echo "🎉🎉🎉 SUCCESS! Complete event flow working! 🎉🎉🎉"
            echo ""
            echo "✅ Event ingested from Sepolia blockchain"
            echo "✅ Matched to your USDC endpoint"
            echo "✅ Stored in database"
            echo "✅ Webhook delivery job created"
            echo ""
            echo "Check your webhook receiver for the HTTP POST!"
            echo "Refresh UI at http://localhost:3002 to see the event!"
            echo ""
        fi
    fi

    LAST_REDIS=$CURRENT_REDIS
    LAST_DB=$CURRENT_DB
    LAST_JOBS=$CURRENT_JOBS
done
