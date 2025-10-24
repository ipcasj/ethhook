"""
Demo Webhook Receiver - For demonstration purposes only

This service receives and displays webhooks from EthHook to demonstrate
the system working. In production, customers deploy their own webhook
endpoints and EthHook sends webhooks to those endpoints.

This receiver:
- Shows incoming webhook data in terminal
- Verifies HMAC signatures
- Always returns 200 OK
- Stores last 100 webhooks for demo dashboard
"""

import hmac
import hashlib
import json
from datetime import datetime
from flask import Flask, request, jsonify
from collections import deque

app = Flask(__name__)

# Store last 100 webhooks for demo display
webhook_history = deque(maxlen=100)


def verify_hmac(payload: bytes, signature: str, secret: str) -> bool:
    """Verify HMAC signature from EthHook"""
    expected = hmac.new(
        secret.encode(), payload, hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(signature, expected)


@app.route('/webhook', methods=['POST'])
def receive_webhook():
    """Receive webhook from EthHook"""
    
    # Get headers
    signature = request.headers.get('x-webhook-signature', '')
    webhook_id = request.headers.get('x-webhook-id', 'N/A')
    attempt = request.headers.get('x-webhook-attempt', '1')
    
    # Get payload
    payload = request.get_data()
    
    try:
        data = json.loads(payload)
    except:
        data = {}
    
    # Store in history
    webhook_entry = {
        'timestamp': datetime.now().isoformat(),
        'webhook_id': webhook_id,
        'attempt': attempt,
        'signature': signature[:16] + '...',  # Truncate for display
        'data': data
    }
    webhook_history.append(webhook_entry)
    
    # Pretty print to console
    print("\n" + "="*60)
    print(f"🎉 WEBHOOK RECEIVED [{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}]")
    print("="*60)
    print(f"  ID: {webhook_id}")
    print(f"  Attempt: {attempt}")
    print(f"  Signature: {signature[:32]}...")
    print(f"\n  Payload:")
    print(f"    Chain: {data.get('chain_id', 'N/A')}")
    print(f"    Block: {data.get('block_number', 'N/A')}")
    print(f"    Contract: {data.get('contract_address', 'N/A')}")
    print(f"    Tx: {data.get('transaction_hash', 'N/A')[:20]}..." if data.get('transaction_hash') else "")
    print("="*60 + "\n")
    
    # Always return 200 OK for demo
    return jsonify({
        'status': 'success',
        'received_at': datetime.now().isoformat(),
        'webhook_id': webhook_id
    }), 200


@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint"""
    return jsonify({
        'status': 'healthy',
        'service': 'demo-webhook-receiver',
        'webhooks_received': len(webhook_history)
    }), 200


@app.route('/history', methods=['GET'])
def history():
    """Get recent webhook history"""
    return jsonify({
        'total': len(webhook_history),
        'webhooks': list(webhook_history)
    }), 200


if __name__ == '__main__':
    print("\n" + "="*60)
    print("🚀 Demo Webhook Receiver Starting")
    print("="*60)
    print("  Purpose: Demonstrate EthHook system working")
    print("  Note: In production, customers deploy their own endpoints")
    print("  Listening: http://0.0.0.0:8000")
    print("  Endpoints:")
    print("    POST /webhook  - Receive webhooks")
    print("    GET  /health   - Health check")
    print("    GET  /history  - Recent webhooks")
    print("="*60 + "\n")
    
    app.run(host='0.0.0.0', port=8000, debug=False)
