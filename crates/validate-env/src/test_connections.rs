//! Test actual connections to verify .env settings work
//! 
//! Run with: cargo run -p validate-env --bin test-connections

use std::env;

#[tokio::main]
async fn main() {
    println!("🧪 Testing EthHook Service Connections...\n");
    
    // Load .env file
    dotenvy::dotenv().ok();
    
    let mut successes = 0;
    let mut failures = 0;
    
    // Test PostgreSQL
    println!("🗄️  Testing PostgreSQL connection...");
    match test_postgres().await {
        Ok(_) => {
            println!("   ✅ PostgreSQL: Connected successfully\n");
            successes += 1;
        }
        Err(e) => {
            println!("   ❌ PostgreSQL: Failed - {}\n", e);
            failures += 1;
        }
    }
    
    // Test Redis
    println!("📦 Testing Redis connection...");
    match test_redis().await {
        Ok(_) => {
            println!("   ✅ Redis: Connected successfully\n");
            successes += 1;
        }
        Err(e) => {
            println!("   ❌ Redis: Failed - {}\n", e);
            failures += 1;
        }
    }
    
    // Summary
    println!("═══════════════════════════════════════════════");
    println!("Results: {} passed, {} failed", successes, failures);
    
    if failures == 0 {
        println!("✅ All services are reachable!");
        std::process::exit(0);
    } else {
        println!("\n❌ Some services are not reachable.");
        println!("💡 Make sure Docker services are running:");
        println!("   docker compose up -d postgres redis");
        std::process::exit(1);
    }
}

async fn test_postgres() -> Result<(), String> {
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL not set".to_string())?;
    
    // Simple connection test (doesn't require sqlx in this binary)
    let parts: Vec<&str> = database_url.split('@').collect();
    if parts.len() < 2 {
        return Err("Invalid DATABASE_URL format".to_string());
    }
    
    println!("   Connecting to: {}", database_url.split('@').last().unwrap());
    
    // Use a simple TCP check
    let host_port = parts[1].trim_start_matches("//");
    if let Some(host_part) = host_port.split('/').next() {
        if let Some((host, port_str)) = host_part.rsplit_once(':') {
            let port: u16 = port_str.parse().map_err(|_| "Invalid port".to_string())?;
            
            match tokio::net::TcpStream::connect((host, port)).await {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Connection failed: {}", e)),
            }
        } else {
            Err("Could not parse host:port".to_string())
        }
    } else {
        Err("Invalid DATABASE_URL format".to_string())
    }
}

async fn test_redis() -> Result<(), String> {
    let redis_url = env::var("REDIS_URL")
        .map_err(|_| "REDIS_URL not set".to_string())?;
    
    println!("   Connecting to: {}", redis_url.trim_start_matches("redis://"));
    
    // Parse redis://host:port
    let url_without_protocol = redis_url.trim_start_matches("redis://");
    if let Some((host, port_str)) = url_without_protocol.rsplit_once(':') {
        let port: u16 = port_str.parse().map_err(|_| "Invalid port".to_string())?;
        
        match tokio::net::TcpStream::connect((host, port)).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Connection failed: {}", e)),
        }
    } else {
        // Default to localhost:6379
        match tokio::net::TcpStream::connect(("localhost", 6379)).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Connection failed: {}", e)),
        }
    }
}
