#!/bin/bash
# Verify EthHook Setup for Real Testing

echo "🔍 EthHook Setup Verification"
echo "=============================="
echo ""

echo "📊 ACTIVE ENDPOINTS:"
/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -t -c "
SELECT
    '  ' || name || ' (' ||
    CASE
        WHEN webhook_url LIKE '%localhost%' OR webhook_url LIKE '%host.docker.internal%' THEN '✅ REAL'
        ELSE '🔴 webhook.site'
    END || ')'
FROM endpoints
WHERE is_active = true
ORDER BY created_at DESC;
"

echo ""
echo "📊 INACTIVE ENDPOINTS:"
/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -t -c "
SELECT
    '  ' || name || ' (deactivated)'
FROM endpoints
WHERE is_active = false
ORDER BY created_at DESC;
"

echo ""
echo "🎯 REAL ENDPOINT DETAILS:"
/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -c "
SELECT
    'ID: ' || id::text as detail
FROM endpoints
WHERE id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
UNION ALL
SELECT 'URL: ' || webhook_url
FROM endpoints
WHERE id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
UNION ALL
SELECT 'Contract: ' || contract_addresses[1]
FROM endpoints
WHERE id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
UNION ALL
SELECT 'Active: ' || CASE WHEN is_active THEN '✅ YES' ELSE '❌ NO' END
FROM endpoints
WHERE id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';
" -t

echo ""
echo "🚀 SERVICES STATUS:"
echo ""

if /usr/local/bin/docker ps | grep -q ethhook-postgres; then
    echo "  ✅ PostgreSQL running"
else
    echo "  ❌ PostgreSQL NOT running"
fi

if /usr/local/bin/docker ps | grep -q ethhook-redis; then
    echo "  ✅ Redis running"
else
    echo "  ❌ Redis NOT running"
fi

if /usr/local/bin/docker ps | grep -q ethhook-event-ingestor; then
    echo "  ✅ Event Ingestor running"
else
    echo "  ❌ Event Ingestor NOT running"
fi

if /usr/local/bin/docker ps | grep -q ethhook-message-processor; then
    echo "  ✅ Message Processor running"
else
    echo "  ❌ Message Processor NOT running"
fi

if /usr/local/bin/docker ps | grep -q ethhook-webhook-delivery; then
    echo "  ✅ Webhook Delivery running"
else
    echo "  ❌ Webhook Delivery NOT running"
fi

if /usr/local/bin/docker ps | grep -q ethhook-admin-api; then
    echo "  ✅ Admin API running"
else
    echo "  ❌ Admin API NOT running"
fi

echo ""
echo "📈 RECENT ACTIVITY:"
echo ""

EVENTS_COUNT=$(/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -t -c "SELECT COUNT(*) FROM events;")
echo "  Events captured: $EVENTS_COUNT"

DELIVERIES=$(/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -t -c "SELECT COUNT(*) FROM delivery_attempts;")
echo "  Deliveries attempted: $DELIVERIES"

SUCCESS=$(/usr/local/bin/docker exec ethhook-postgres psql -U ethhook -d ethhook -t -c "SELECT COUNT(*) FROM delivery_attempts WHERE success = true;")
echo "  Successful deliveries: $SUCCESS"

echo ""
echo "✅ READY TO TEST!"
echo ""
echo "Next step: ./test_real_webhooks.sh"
echo ""
