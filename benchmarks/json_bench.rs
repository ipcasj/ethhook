// JSON Parsing Benchmark - Rust Implementation
use std::time::Instant;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct EventData {
    from: String,
    to: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    id: u64,
    chain_id: u64,
    block_number: u64,
    transaction_hash: String,
    contract_address: String,
    event_name: String,
    event_data: EventData,
    timestamp: u64,
}

const SAMPLE_EVENT: &str = r#"{
  "id": 12345,
  "chain_id": 1,
  "block_number": 17000000,
  "transaction_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
  "contract_address": "0xabcdef1234567890abcdef1234567890abcdef12",
  "event_name": "Transfer",
  "event_data": {
    "from": "0x0000000000000000000000000000000000000000",
    "to": "0xabcdef1234567890abcdef1234567890abcdef12",
    "value": "1000000000000000000"
  },
  "timestamp": 1638360000
}"#;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let iterations = if args.len() > 1 {
        args[1].parse().unwrap_or(10_000)
    } else {
        10_000
    };
    
    let start = Instant::now();
    
    let mut parsed = 0;
    for _ in 0..iterations {
        if let Ok(event) = serde_json::from_str::<Event>(SAMPLE_EVENT) {
            // Access some fields
            let _ = event.id;
            let _ = event.chain_id;
            let _ = event.block_number;
            parsed += 1;
        }
    }
    
    let elapsed = start.elapsed();
    let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
    
    println!("JSON Parsing Benchmark (Rust - serde_json)");
    println!("Iterations: {}", iterations);
    println!("Successfully parsed: {}", parsed);
    println!("Total time: {:.3} seconds", elapsed.as_secs_f64());
    println!("Operations/sec: {:.0}", ops_per_sec);
    println!("Time per operation: {:.3} µs", (elapsed.as_secs_f64() / iterations as f64) * 1e6);
}
