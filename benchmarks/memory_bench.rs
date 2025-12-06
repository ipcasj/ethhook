// Memory Usage Benchmark - Rust Implementation
use std::time::Instant;

#[derive(Clone)]
struct Record {
    id: usize,
    data: String,
    timestamp: Instant,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let count = if args.len() > 1 {
        args[1].parse().unwrap_or(100_000)
    } else {
        100_000
    };
    
    // Allocate vector
    let mut records: Vec<Record> = Vec::with_capacity(count);
    
    // Initialize records
    for i in 0..count {
        records.push(Record {
            id: i,
            data: format!("Record {} with some data", i),
            timestamp: Instant::now(),
        });
    }
    
    // Process records (simulate some work)
    let sum: usize = records.iter().map(|r| r.id).sum();
    
    println!("Memory Usage Benchmark (Rust)");
    println!("Records allocated: {}", count);
    println!("Record size: ~{} bytes (estimated)", std::mem::size_of::<Record>());
    println!("Vector capacity: {}", records.capacity());
    println!("Sum check: {}", sum);
    
    // Note: Rust doesn't expose RSS directly without external crates
    // This benchmark focuses on allocation patterns
}
