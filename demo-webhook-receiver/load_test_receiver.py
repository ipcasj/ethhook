"""
Load Test Webhook Receiver - High-performance metrics collection

This receiver is optimized for load testing:
- Measures end-to-end latency
- Tracks throughput (requests/second)
- Monitors error rates
- Collects percentile statistics (p50, p95, p99)
- Minimal processing overhead
"""

import hmac
import hashlib
import json
import time
from datetime import datetime
from flask import Flask, request, jsonify
from collections import deque
from threading import Lock
import statistics

app = Flask(__name__)

# Performance metrics
class PerformanceMetrics:
    def __init__(self):
        self.lock = Lock()
        self.total_requests = 0
        self.successful_requests = 0
        self.failed_requests = 0
        self.latencies = deque(maxlen=10000)  # Keep last 10k for percentiles
        self.start_time = time.time()
        self.last_report_time = time.time()
        self.requests_since_last_report = 0

    def record_request(self, latency_ms: float, success: bool):
        with self.lock:
            self.total_requests += 1
            self.requests_since_last_report += 1
            if success:
                self.successful_requests += 1
            else:
                self.failed_requests += 1
            self.latencies.append(latency_ms)

    def get_stats(self):
        with self.lock:
            elapsed = time.time() - self.start_time
            throughput = self.total_requests / elapsed if elapsed > 0 else 0

            # Calculate recent throughput (last interval)
            time_since_report = time.time() - self.last_report_time
            recent_throughput = (
                self.requests_since_last_report / time_since_report
                if time_since_report > 0 else 0
            )

            latencies_list = list(self.latencies)

            stats = {
                'total_requests': self.total_requests,
                'successful': self.successful_requests,
                'failed': self.failed_requests,
                'uptime_seconds': round(elapsed, 2),
                'throughput_rps': round(throughput, 2),
                'recent_throughput_rps': round(recent_throughput, 2),
                'latency_ms': {}
            }

            if latencies_list:
                sorted_latencies = sorted(latencies_list)
                stats['latency_ms'] = {
                    'min': round(min(latencies_list), 2),
                    'max': round(max(latencies_list), 2),
                    'avg': round(statistics.mean(latencies_list), 2),
                    'median': round(statistics.median(latencies_list), 2),
                    'p95': round(sorted_latencies[int(len(sorted_latencies) * 0.95)], 2),
                    'p99': round(sorted_latencies[int(len(sorted_latencies) * 0.99)], 2),
                }

            return stats

    def reset_interval(self):
        with self.lock:
            self.last_report_time = time.time()
            self.requests_since_last_report = 0

metrics = PerformanceMetrics()


@app.route('/webhook', methods=['POST'])
def receive_webhook():
    """Receive webhook and measure latency"""
    request_start = time.time()

    try:
        # Get timestamp from webhook payload
        data = request.get_json()

        # Calculate end-to-end latency using timestamp field
        timestamp = data.get('timestamp')
        if timestamp:
            # timestamp is Unix epoch seconds
            latency_ms = (request_start - timestamp) * 1000
        else:
            # Just measure processing time if no timestamp
            latency_ms = 0

        metrics.record_request(latency_ms, success=True)

        return jsonify({
            'status': 'success',
            'latency_ms': round(latency_ms, 2)
        }), 200

    except Exception as e:
        metrics.record_request(0, success=False)
        return jsonify({
            'status': 'error',
            'error': str(e)
        }), 500


@app.route('/metrics', methods=['GET'])
def get_metrics():
    """Get performance metrics"""
    stats = metrics.get_stats()
    return jsonify(stats), 200


@app.route('/metrics/reset', methods=['POST'])
def reset_metrics():
    """Reset metrics counters"""
    global metrics
    metrics = PerformanceMetrics()
    return jsonify({'status': 'reset'}), 200


@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint"""
    stats = metrics.get_stats()
    return jsonify({
        'status': 'healthy',
        'service': 'load-test-webhook-receiver',
        'total_requests': stats['total_requests'],
        'throughput_rps': stats['throughput_rps']
    }), 200


if __name__ == '__main__':
    print("\n" + "="*70)
    print("🚀 Load Test Webhook Receiver Starting")
    print("="*70)
    print("  Purpose: High-performance metrics collection for load testing")
    print("  Listening: http://0.0.0.0:8000")
    print("  Endpoints:")
    print("    POST /webhook         - Receive webhooks (measures latency)")
    print("    GET  /metrics         - Performance statistics")
    print("    POST /metrics/reset   - Reset counters")
    print("    GET  /health          - Health check")
    print("="*70)
    print("\n📊 Metrics tracked:")
    print("  - Total requests")
    print("  - Success/failure rate")
    print("  - Throughput (requests/second)")
    print("  - Latency percentiles (min/avg/p50/p95/p99/max)")
    print("="*70 + "\n")

    # Run with multiple workers for high throughput
    app.run(host='0.0.0.0', port=8000, debug=False, threaded=True)
