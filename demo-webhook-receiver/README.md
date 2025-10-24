# Demo Webhook Receiver

## ⚠️ For Demonstration Purposes Only

This service is included to demonstrate EthHook working end-to-end in your demo deployment.

## Production vs Demo

### In Production

- **Customers deploy their own webhook endpoints** (e.g., their backend services)
- EthHook **sends** webhooks to customer endpoints
- EthHook does **not** receive webhooks
- Customers write code in their preferred language (Python, Node.js, Go, etc.)

### In Demo

- This receiver shows the system working
- Displays incoming webhooks in real-time
- Verifies HMAC signatures
- Always returns 200 OK

## Endpoints

- `POST /webhook` - Receives webhooks from EthHook
- `GET /health` - Health check
- `GET /history` - View last 100 webhooks

## Environment Variables

None required - it just works!

## Running Standalone

```bash
cd demo-webhook-receiver
python receiver.py
```

## Running with Docker

```bash
docker build -t demo-webhook-receiver .
docker run -p 8000:8000 demo-webhook-receiver
```

## Testing

Send a test webhook:

```bash
curl -X POST http://localhost:8000/webhook \
  -H "Content-Type: application/json" \
  -H "x-webhook-id: test-123" \
  -H "x-webhook-signature: abc123" \
  -d '{
    "chain_id": 1,
    "block_number": 12345,
    "contract_address": "0x1234567890",
    "transaction_hash": "0xabcdef"
  }'
```

View history:

```bash
curl http://localhost:8000/history
```

## For Customers

See `docs/CUSTOMER_INTEGRATION_GUIDE.md` for how to build your own webhook receiver in your preferred language.
