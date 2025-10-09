//! Validates .env configuration file for EthHook
//!
//! Run with: cargo run -p validate-env

use std::env;

fn main() {
    println!("🔍 Validating EthHook Configuration...\n");

    // Load .env file
    if let Err(e) = dotenvy::dotenv() {
        eprintln!("⚠️  Warning: Could not load .env file: {}", e);
        eprintln!("    Make sure .env file exists in the project root\n");
    }

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // ========== RPC Provider Validation ==========
    println!("📡 RPC Providers:");

    validate_rpc_endpoint(
        &mut errors,
        &mut warnings,
        "ETH_RPC_WS",
        "Ethereum WebSocket (primary)",
    );
    validate_rpc_endpoint(
        &mut errors,
        &mut warnings,
        "ETH_RPC_HTTP",
        "Ethereum HTTP (primary)",
    );
    validate_optional_rpc(
        &mut warnings,
        "ETH_RPC_WS_BACKUP",
        "Ethereum WebSocket (backup)",
    );
    validate_optional_rpc(
        &mut warnings,
        "ETH_RPC_HTTP_BACKUP",
        "Ethereum HTTP (backup)",
    );

    validate_rpc_endpoint(
        &mut errors,
        &mut warnings,
        "ARBITRUM_RPC_WS",
        "Arbitrum WebSocket (primary)",
    );
    validate_rpc_endpoint(
        &mut errors,
        &mut warnings,
        "ARBITRUM_RPC_HTTP",
        "Arbitrum HTTP (primary)",
    );
    validate_optional_rpc(
        &mut warnings,
        "ARBITRUM_RPC_WS_BACKUP",
        "Arbitrum WebSocket (backup)",
    );
    validate_optional_rpc(
        &mut warnings,
        "ARBITRUM_RPC_HTTP_BACKUP",
        "Arbitrum HTTP (backup)",
    );

    validate_rpc_endpoint(
        &mut errors,
        &mut warnings,
        "OPTIMISM_RPC_WS",
        "Optimism WebSocket (primary)",
    );
    validate_rpc_endpoint(
        &mut errors,
        &mut warnings,
        "OPTIMISM_RPC_HTTP",
        "Optimism HTTP (primary)",
    );
    validate_optional_rpc(
        &mut warnings,
        "OPTIMISM_RPC_WS_BACKUP",
        "Optimism WebSocket (backup)",
    );
    validate_optional_rpc(
        &mut warnings,
        "OPTIMISM_RPC_HTTP_BACKUP",
        "Optimism HTTP (backup)",
    );

    validate_rpc_endpoint(
        &mut errors,
        &mut warnings,
        "BASE_RPC_WS",
        "Base WebSocket (primary)",
    );
    validate_rpc_endpoint(
        &mut errors,
        &mut warnings,
        "BASE_RPC_HTTP",
        "Base HTTP (primary)",
    );
    validate_optional_rpc(
        &mut warnings,
        "BASE_RPC_WS_BACKUP",
        "Base WebSocket (backup)",
    );
    validate_optional_rpc(&mut warnings, "BASE_RPC_HTTP_BACKUP", "Base HTTP (backup)");

    println!();

    // ========== Database Validation ==========
    println!("🗄️  Database:");
    validate_required(&mut errors, "DATABASE_URL", "PostgreSQL connection string");
    validate_numeric(&mut errors, "DATABASE_MAX_CONNECTIONS", 1, 100, Some(20));
    validate_numeric(&mut errors, "DATABASE_MIN_CONNECTIONS", 1, 50, Some(5));
    println!();

    // ========== Redis Validation ==========
    println!("📦 Redis:");
    validate_redis(&mut errors, "REDIS_URL");
    validate_numeric(&mut errors, "REDIS_POOL_SIZE", 1, 100, Some(10));
    println!();

    // ========== API Configuration ==========
    println!("🌐 API Server:");
    validate_optional(&mut warnings, "API_HOST", "0.0.0.0");
    validate_numeric(&mut errors, "API_PORT", 1024, 65535, Some(8080));
    validate_jwt_secret(&mut errors);
    validate_numeric(&mut errors, "JWT_EXPIRATION_HOURS", 1, 168, Some(24));
    validate_numeric(
        &mut errors,
        "API_RATE_LIMIT_PER_MINUTE",
        1,
        10000,
        Some(100),
    );
    println!();

    // ========== Webhook Configuration ==========
    println!("🪝 Webhook Delivery:");
    validate_numeric(&mut errors, "WEBHOOK_TIMEOUT_SECONDS", 1, 300, Some(30));
    validate_numeric(&mut errors, "WEBHOOK_MAX_RETRIES", 0, 10, Some(5));
    validate_numeric(&mut errors, "WEBHOOK_WORKER_THREADS", 1, 100, Some(10));
    println!();

    // ========== Observability ==========
    println!("📊 Observability:");
    validate_optional(&mut warnings, "RUST_LOG", "info,ethhook=debug");
    validate_optional(
        &mut warnings,
        "OTEL_EXPORTER_OTLP_ENDPOINT",
        "http://localhost:4317",
    );
    validate_numeric(
        &mut errors,
        "PROMETHEUS_METRICS_PORT",
        1024,
        65535,
        Some(9090),
    );
    println!();

    // ========== Summary ==========
    println!("═══════════════════════════════════════════════");

    if errors.is_empty() && warnings.is_empty() {
        println!("✅ Configuration is valid! All required settings are present.");
        std::process::exit(0);
    }

    if !warnings.is_empty() {
        println!("\n⚠️  Warnings ({}):", warnings.len());
        for warning in &warnings {
            println!("   - {}", warning);
        }
    }

    if !errors.is_empty() {
        println!("\n❌ Errors ({}):", errors.len());
        for error in &errors {
            println!("   - {}", error);
        }
        println!("\n💡 Fix these errors before running EthHook services.");
        std::process::exit(1);
    }

    if !warnings.is_empty() {
        println!(
            "\n✅ Configuration is valid (with {} warnings)",
            warnings.len()
        );
        std::process::exit(0);
    }
}

fn validate_required(errors: &mut Vec<String>, key: &str, description: &str) {
    match env::var(key) {
        Ok(value) if !value.trim().is_empty() => {
            println!("  ✓ {}: {}", description, mask_sensitive(key, &value));
        }
        Ok(_) => {
            errors.push(format!("{} is set but empty", key));
            println!("  ✗ {}: EMPTY", description);
        }
        Err(_) => {
            errors.push(format!("{} is required but not set", key));
            println!("  ✗ {}: NOT SET", description);
        }
    }
}

fn validate_optional(warnings: &mut Vec<String>, key: &str, default: &str) {
    match env::var(key) {
        Ok(value) => {
            println!("  ✓ {}: {}", key, value);
        }
        Err(_) => {
            warnings.push(format!("{} not set, will use default: {}", key, default));
            println!("  ⚠ {}: using default ({})", key, default);
        }
    }
}

fn validate_numeric(errors: &mut Vec<String>, key: &str, min: u32, max: u32, default: Option<u32>) {
    match env::var(key) {
        Ok(value) => match value.parse::<u32>() {
            Ok(num) if num >= min && num <= max => {
                println!("  ✓ {}: {}", key, num);
            }
            Ok(num) => {
                errors.push(format!(
                    "{} value {} is out of range ({}-{})",
                    key, num, min, max
                ));
                println!("  ✗ {}: {} (out of range {}-{})", key, num, min, max);
            }
            Err(_) => {
                errors.push(format!("{} must be a number, got: {}", key, value));
                println!("  ✗ {}: {} (not a number)", key, value);
            }
        },
        Err(_) => {
            if let Some(def) = default {
                println!("  ⚠ {}: using default ({})", key, def);
            } else {
                errors.push(format!("{} is required but not set", key));
                println!("  ✗ {}: NOT SET", key);
            }
        }
    }
}

fn validate_rpc_endpoint(
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
    key: &str,
    description: &str,
) {
    match env::var(key) {
        Ok(value) if value.starts_with("wss://") || value.starts_with("https://") => {
            if value.contains("YOUR_KEY") || value.contains("YOUR_PROJECT_ID") {
                warnings.push(format!("{} contains placeholder API key", key));
                println!(
                    "  ⚠ {}: {} (contains placeholder)",
                    description,
                    mask_url(&value)
                );
            } else {
                println!("  ✓ {}: {}", description, mask_url(&value));
            }
        }
        Ok(value) => {
            errors.push(format!(
                "{} must start with wss:// or https://, got: {}",
                key, value
            ));
            println!("  ✗ {}: invalid protocol", description);
        }
        Err(_) => {
            errors.push(format!("{} is required", key));
            println!("  ✗ {}: NOT SET", description);
        }
    }
}

fn validate_optional_rpc(warnings: &mut Vec<String>, key: &str, description: &str) {
    match env::var(key) {
        Ok(value) if value.starts_with("wss://") || value.starts_with("https://") => {
            println!("  ✓ {}: {}", description, mask_url(&value));
        }
        Ok(_) => {
            warnings.push(format!("{} is set but has invalid protocol", key));
            println!("  ⚠ {}: invalid protocol", description);
        }
        Err(_) => {
            warnings.push(format!("{} not set (backup RPC recommended)", key));
            println!("  ⚠ {}: not set (backup recommended)", description);
        }
    }
}

fn validate_redis(errors: &mut Vec<String>, key: &str) {
    match env::var(key) {
        Ok(value) if value.starts_with("redis://") || value.starts_with("rediss://") => {
            println!("  ✓ REDIS_URL: {}", mask_url(&value));
        }
        Ok(value) => {
            errors.push(format!(
                "{} must start with redis:// or rediss://, got: {}",
                key, value
            ));
            println!("  ✗ REDIS_URL: invalid protocol");
        }
        Err(_) => {
            errors.push(format!("{} is required", key));
            println!("  ✗ REDIS_URL: NOT SET");
        }
    }
}

fn validate_jwt_secret(errors: &mut Vec<String>) {
    match env::var("JWT_SECRET") {
        Ok(value) if value.len() >= 32 => {
            println!(
                "  ✓ JWT_SECRET: {} (length: {})",
                mask_secret(&value),
                value.len()
            );
        }
        Ok(value) => {
            errors.push(format!(
                "JWT_SECRET must be at least 32 characters, got {}",
                value.len()
            ));
            println!(
                "  ✗ JWT_SECRET: too short ({} chars, need 32+)",
                value.len()
            );
        }
        Err(_) => {
            errors.push("JWT_SECRET is required".to_string());
            println!("  ✗ JWT_SECRET: NOT SET");
        }
    }
}

fn mask_sensitive(key: &str, value: &str) -> String {
    if key.contains("SECRET") || key.contains("PASSWORD") {
        mask_secret(value)
    } else {
        value.to_string()
    }
}

fn mask_secret(value: &str) -> String {
    if value.len() <= 8 {
        "***".to_string()
    } else {
        format!("{}...{}", &value[..4], &value[value.len() - 4..])
    }
}

fn mask_url(url: &str) -> String {
    // Mask API key in URL (after /v2/ or /v3/)
    if let Some(pos) = url.rfind("/v2/").or_else(|| url.rfind("/v3/")) {
        let (prefix, suffix) = url.split_at(pos + 4);
        if suffix.len() > 10 {
            format!(
                "{}{}...{}",
                prefix,
                &suffix[..5],
                &suffix[suffix.len() - 3..]
            )
        } else {
            format!("{}***", prefix)
        }
    } else {
        url.to_string()
    }
}
