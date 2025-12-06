// JWT Benchmark - Rust Implementation
use std::time::Instant;
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    name: String,
    iat: usize,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let iterations = if args.len() > 1 {
        args[1].parse().unwrap_or(100_000)
    } else {
        100_000
    };
    
    let secret = "your-256-bit-secret";
    let claims = Claims {
        sub: "1234567890".to_string(),
        name: "John Doe".to_string(),
        iat: 1516239022,
    };
    
    let mut header = Header::default();
    header.alg = Algorithm::HS256;
    
    let key = EncodingKey::from_secret(secret.as_bytes());
    
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _ = encode(&header, &claims, &key).unwrap();
    }
    
    let elapsed = start.elapsed();
    let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
    
    println!("JWT Signing Benchmark (Rust)");
    println!("Iterations: {}", iterations);
    println!("Total time: {:.3} seconds", elapsed.as_secs_f64());
    println!("Operations/sec: {:.0}", ops_per_sec);
    println!("Time per operation: {:.3} µs", (elapsed.as_secs_f64() / iterations as f64) * 1e6);
}
