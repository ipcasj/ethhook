#!/usr/bin/env python3
"""
Real Webhook Receiver for EthHook Testing
Receives actual webhook deliveries and displays them in real-time
"""

from http.server import HTTPServer, BaseHTTPRequestHandler
from datetime import datetime
import json
import hmac
import hashlib

class WebhookReceiver(BaseHTTPRequestHandler):
    """Handle incoming webhook POST requests"""

    # Store received webhooks
    webhooks_received = []

    def do_POST(self):
        """Handle POST request (webhook delivery)"""

        # Get request details
        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length)

        # Parse JSON body
        try:
            payload = json.loads(body.decode('utf-8'))
        except:
            payload = {"raw": body.decode('utf-8', errors='ignore')}

        # Get headers
        headers = dict(self.headers)

        # Log receipt
        timestamp = datetime.now().strftime('%Y-%m-%d %H:%M:%S')

        print("\n" + "="*80)
        print(f"🎉 WEBHOOK RECEIVED! [{timestamp}]")
        print("="*80)

        # Display headers
        print("\n📋 HEADERS:")
        for key, value in headers.items():
            if key.lower().startswith('x-'):
                print(f"  {key}: {value}")

        # Display payload
        print("\n📦 PAYLOAD:")
        print(json.dumps(payload, indent=2))

        # Verify HMAC signature if present
        signature_header = headers.get('X-Webhook-Signature', '')
        timestamp_header = headers.get('X-Webhook-Timestamp', '')

        if signature_header and timestamp_header:
            print("\n🔒 HMAC SIGNATURE VERIFICATION:")
            print(f"  Signature: {signature_header}")
            print(f"  Timestamp: {timestamp_header}")
            print("  ✅ Signature present (verification requires endpoint secret)")

        # Display event details
        if isinstance(payload, dict):
            print("\n🔍 EVENT DETAILS:")
            print(f"  Chain ID: {payload.get('chain_id', 'N/A')}")
            print(f"  Chain Name: {payload.get('chain_name', 'N/A')}")
            print(f"  Block: {payload.get('block_number', 'N/A')}")
            print(f"  Transaction: {payload.get('transaction_hash', 'N/A')}")
            print(f"  Contract: {payload.get('contract_address', 'N/A')}")
            print(f"  Event: {payload.get('event_signature', 'N/A')}")

            # Decoded data
            decoded = payload.get('decoded', {})
            if decoded:
                print(f"\n  📊 DECODED DATA:")
                for key, value in decoded.items():
                    print(f"    {key}: {value}")

        print("\n" + "="*80)
        print(f"✅ Total webhooks received: {len(self.webhooks_received) + 1}")
        print("="*80 + "\n")

        # Store webhook
        self.webhooks_received.append({
            'timestamp': timestamp,
            'headers': headers,
            'payload': payload
        })

        # Send 200 OK response
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        response = {'status': 'received', 'timestamp': timestamp}
        self.wfile.write(json.dumps(response).encode())

    def do_GET(self):
        """Handle GET request (health check)"""
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()

        response = {
            'status': 'running',
            'webhooks_received': len(self.webhooks_received),
            'message': 'Webhook receiver is ready!'
        }
        self.wfile.write(json.dumps(response, indent=2).encode())

    def log_message(self, format, *args):
        """Suppress default HTTP logging (we have custom logging)"""
        pass


def run_server(port=8000):
    """Start the webhook receiver server"""

    server_address = ('', port)
    httpd = HTTPServer(server_address, WebhookReceiver)

    print("\n" + "="*80)
    print("🚀 REAL WEBHOOK RECEIVER STARTED!")
    print("="*80)
    print(f"\n📍 Listening on: http://0.0.0.0:{port}")
    print(f"📍 Webhook URL:  http://localhost:{port}/webhook")
    print(f"📍 Public URL:   http://YOUR_IP:{port}/webhook")
    print("\n💡 Use this URL in your EthHook endpoint configuration")
    print("\n⏳ Waiting for webhooks from EthHook...")
    print("   (Press Ctrl+C to stop)\n")
    print("="*80 + "\n")

    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\n\n" + "="*80)
        print("🛑 WEBHOOK RECEIVER STOPPED")
        print("="*80)
        print(f"\n📊 Total webhooks received: {len(WebhookReceiver.webhooks_received)}")
        print("\nThank you for testing EthHook! 🎉\n")


if __name__ == '__main__':
    import sys

    # Get port from command line or use default
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8000

    run_server(port)
