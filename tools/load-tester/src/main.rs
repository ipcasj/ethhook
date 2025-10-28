use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use colored::Colorize;
use hdrhistogram::Histogram;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// EthHook Load Testing Tool
///
/// Simulates high-traffic blockchain events to test system performance
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of events to generate
    #[arg(short = 'n', long, default_value = "10000")]
    events: usize,

    /// Events per second (0 = unlimited)
    #[arg(short = 'r', long, default_value = "1000")]
    rate: usize,

    /// Number of concurrent publishers
    #[arg(short = 'c', long, default_value = "10")]
    concurrency: usize,

    /// Redis URL
    #[arg(long, default_value = "redis://localhost:6379")]
    redis_url: String,

    /// Chain ID to simulate
    #[arg(long, default_value = "11155111")]
    chain_id: u64,

    /// Webhook receiver metrics endpoint
    #[arg(long, default_value = "http://localhost:8000/metrics")]
    metrics_url: String,

    /// Duration to run test (seconds, 0 = use event count)
    #[arg(short = 'd', long, default_value = "0")]
    duration: u64,
}

#[derive(Clone)]
struct LoadTestMetrics {
    events_published: Arc<Mutex<usize>>,
    errors: Arc<Mutex<usize>>,
    latencies: Arc<Mutex<Histogram<u64>>>,
}

impl LoadTestMetrics {
    fn new() -> Self {
        Self {
            events_published: Arc::new(Mutex::new(0)),
            errors: Arc::new(Mutex::new(0)),
            latencies: Arc::new(Mutex::new(
                Histogram::<u64>::new_with_bounds(1, 60_000, 3).unwrap(),
            )),
        }
    }

    async fn record_event(&self, latency_ms: u64) {
        *self.events_published.lock().await += 1;
        let _ = self.latencies.lock().await.record(latency_ms);
    }

    async fn record_error(&self) {
        *self.errors.lock().await += 1;
    }

    async fn get_summary(&self) -> (usize, usize) {
        let published = *self.events_published.lock().await;
        let errors = *self.errors.lock().await;
        (published, errors)
    }

    async fn get_latency_stats(&self) -> String {
        let hist = self.latencies.lock().await;
        if hist.is_empty() {
            return "No data".to_string();
        }

        format!(
            "min: {}ms, p50: {}ms, p95: {}ms, p99: {}ms, max: {}ms",
            hist.min(),
            hist.value_at_quantile(0.5),
            hist.value_at_quantile(0.95),
            hist.value_at_quantile(0.99),
            hist.max()
        )
    }
}

async fn publish_event(
    redis: &mut redis::aio::MultiplexedConnection,
    chain_id: u64,
    block_number: u64,
    metrics: &LoadTestMetrics,
) -> Result<()> {
    let start = Instant::now();

    // Generate realistic event data matching event-ingestor format
    let topics_json = json!([
        "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
        "0x000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266",
        "0x00000000000000000000000070997970c51812dc3a010c7d01b50e0d17dc79c8"
    ])
    .to_string();

    let stream_key = format!("events:{chain_id}");

    // Publish to Redis stream using same format as event-ingestor
    // Uses individual fields, not a JSON blob
    redis::cmd("XADD")
        .arg(&stream_key)
        .arg("*")
        .arg("chain_id")
        .arg(chain_id.to_string())
        .arg("block_number")
        .arg(block_number.to_string())
        .arg("block_hash")
        .arg(format!(
            "0x{}",
            hex::encode(uuid::Uuid::new_v4().as_bytes())
        ))
        .arg("tx_hash")
        .arg(format!(
            "0x{}",
            hex::encode(uuid::Uuid::new_v4().as_bytes())
        ))
        .arg("log_index")
        .arg("0")
        .arg("contract")
        .arg("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
        .arg("topics")
        .arg(topics_json)
        .arg("data")
        .arg("0x0000000000000000000000000000000000000000000000000de0b6b3a7640000")
        .arg("timestamp")
        .arg(Utc::now().timestamp().to_string())
        .query_async::<_, ()>(redis)
        .await?;

    let latency = start.elapsed().as_millis() as u64;
    metrics.record_event(latency).await;

    Ok(())
}

async fn run_load_test(args: Args) -> Result<()> {
    println!("\n{}", "=".repeat(70).bright_cyan());
    println!("{}", "🚀 EthHook Load Testing Tool".bright_cyan().bold());
    println!("{}", "=".repeat(70).bright_cyan());
    println!("  Configuration:");
    println!("    Events: {} ({} events/sec)", args.events, args.rate);
    println!("    Concurrency: {} publishers", args.concurrency);
    println!("    Chain ID: {}", args.chain_id);
    println!("    Redis: {}", args.redis_url);
    if args.duration > 0 {
        println!("    Duration: {}s", args.duration);
    }
    println!("{}\n", "=".repeat(70).bright_cyan());

    // Connect to Redis
    let client = redis::Client::open(args.redis_url.as_str())?;
    let mut conn = client.get_multiplexed_async_connection().await?;

    // Check Redis streams exist
    println!("🔍 Checking Redis streams...");
    let stream_key = format!("events:{}", args.chain_id);
    let exists: bool = redis::cmd("EXISTS")
        .arg(&stream_key)
        .query_async(&mut conn)
        .await?;

    if !exists {
        println!("⚠️  Stream {stream_key} doesn't exist, creating it...");
    }

    println!("✅ Redis connected\n");

    // Reset receiver metrics
    println!("📊 Resetting receiver metrics...");
    let client_http = reqwest::Client::new();
    let _ = client_http
        .post(args.metrics_url.replace("/metrics", "/metrics/reset"))
        .send()
        .await;
    println!("✅ Metrics reset\n");

    // Initialize metrics
    let metrics = LoadTestMetrics::new();

    // Calculate timing
    let events_per_publisher = args.events / args.concurrency;
    let delay_per_event = if args.rate > 0 {
        Duration::from_micros(1_000_000 / (args.rate / args.concurrency) as u64)
    } else {
        Duration::from_micros(0)
    };

    // Progress bar
    let progress = ProgressBar::new(args.events as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({per_sec}) {msg}",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    println!("🔥 Starting load test...\n");
    let test_start = Instant::now();

    // Spawn concurrent publishers
    let mut handles = vec![];

    for publisher_id in 0..args.concurrency {
        let mut redis_conn = client.get_multiplexed_async_connection().await?;
        let metrics_clone = metrics.clone();
        let progress_clone = progress.clone();
        let chain_id = args.chain_id;

        let handle = tokio::spawn(async move {
            let mut block_number = 1000000 + (publisher_id * 10000) as u64;

            for _ in 0..events_per_publisher {
                block_number += 1;

                match publish_event(&mut redis_conn, chain_id, block_number, &metrics_clone).await {
                    Ok(_) => {
                        progress_clone.inc(1);
                    }
                    Err(e) => {
                        metrics_clone.record_error().await;
                        eprintln!("Error publishing event: {e}");
                    }
                }

                if delay_per_event.as_micros() > 0 {
                    tokio::time::sleep(delay_per_event).await;
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all publishers to complete
    for handle in handles {
        handle.await?;
    }

    progress.finish_with_message("Complete!");

    let test_duration = test_start.elapsed();

    // Get final metrics
    let (published, errors) = metrics.get_summary().await;
    let latency_stats = metrics.get_latency_stats().await;

    // Wait a bit for webhooks to be delivered
    println!("\n⏳ Waiting 5s for webhook delivery to complete...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Fetch receiver metrics
    let receiver_metrics = client_http.get(&args.metrics_url).send().await?;
    let receiver_stats: serde_json::Value = receiver_metrics.json().await?;

    // Print results
    println!("\n{}", "=".repeat(70).bright_green());
    println!("{}", "📊 Load Test Results".bright_green().bold());
    println!("{}", "=".repeat(70).bright_green());
    println!("\n{}", "Event Publishing:".bright_yellow().bold());
    println!("  Total Events: {published}");
    println!("  Errors: {errors}");
    println!("  Duration: {:.2}s", test_duration.as_secs_f64());
    println!(
        "  Throughput: {:.2} events/sec",
        published as f64 / test_duration.as_secs_f64()
    );
    println!("  Publish Latency: {latency_stats}");

    println!("\n{}", "Webhook Delivery:".bright_yellow().bold());
    println!("  Total Delivered: {}", receiver_stats["total_requests"]);
    println!(
        "  Success Rate: {:.2}%",
        (receiver_stats["successful"].as_u64().unwrap_or(0) as f64
            / receiver_stats["total_requests"].as_u64().unwrap_or(1) as f64)
            * 100.0
    );

    if let Some(latency) = receiver_stats.get("latency_ms") {
        println!("  End-to-End Latency:");
        let min = &latency["min"];
        let avg = &latency["avg"];
        let median = &latency["median"];
        let p95 = &latency["p95"];
        let p99 = &latency["p99"];
        let max = &latency["max"];
        println!("    Min: {min}ms");
        println!("    Avg: {avg}ms");
        println!("    Median: {median}ms");
        println!("    P95: {p95}ms");
        println!("    P99: {p99}ms");
        println!("    Max: {max}ms");
    }

    println!("\n{}", "System Throughput:".bright_yellow().bold());
    println!(
        "  Overall: {:.2} webhooks/sec",
        receiver_stats["throughput_rps"]
    );

    println!("\n{}", "=".repeat(70).bright_green());

    // Performance verdict
    println!("\n{}", "🎯 Performance Analysis:".bright_magenta().bold());

    let avg_latency = receiver_stats["latency_ms"]["avg"]
        .as_f64()
        .unwrap_or(999999.0);
    let throughput = receiver_stats["throughput_rps"].as_f64().unwrap_or(0.0);

    if avg_latency < 500.0 {
        println!("  ✅ Latency: {} (Target: <500ms)", "PASS".bright_green());
    } else {
        println!(
            "  ❌ Latency: {} (Target: <500ms, Actual: {:.0}ms)",
            "FAIL".bright_red(),
            avg_latency
        );
    }

    if throughput >= 1000.0 {
        println!(
            "  ✅ Throughput: {} ({:.0} webhooks/sec)",
            "EXCELLENT".bright_green(),
            throughput
        );
    } else if throughput >= 500.0 {
        println!(
            "  ⚠️  Throughput: {} ({:.0} webhooks/sec)",
            "GOOD".bright_yellow(),
            throughput
        );
    } else {
        println!(
            "  ❌ Throughput: {} ({:.0} webhooks/sec)",
            "NEEDS IMPROVEMENT".bright_red(),
            throughput
        );
    }

    println!("\n");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    run_load_test(args).await
}
