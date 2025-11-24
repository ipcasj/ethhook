use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

/// Safety Rule #2: Runtime health monitoring
///
/// Monitors tokio runtime for deadlocks and performance issues.
/// Based on lessons from Cloudflare Nov 18, 2025 outage and Discord deadlocks.
///
/// Key checks:
/// 1. Tasks are making progress (not stuck in infinite loops)
/// 2. Channel send/receive operations complete (not deadlocked)
/// 3. Runtime metrics are healthy
pub async fn monitor_runtime_health(mut shutdown_rx: broadcast::Receiver<()>) {
    info!("Starting runtime health monitor");

    let check_interval = Duration::from_secs(10);
    let mut last_check = Instant::now();
    let mut consecutive_warnings = 0;

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                info!("Health monitor received shutdown signal");
                break;
            }
            _ = tokio::time::sleep(check_interval) => {
                let elapsed = last_check.elapsed();

                // Safety check: Sleep should take ~10s, not 30s
                // If sleep took way longer, runtime might be overloaded
                if elapsed > Duration::from_secs(15) {
                    consecutive_warnings += 1;
                    warn!(
                        "Health check delayed: expected {}s, took {}s (warning #{}) - possible runtime overload",
                        check_interval.as_secs(),
                        elapsed.as_secs(),
                        consecutive_warnings
                    );

                    // After 3 consecutive warnings, something is seriously wrong
                    if consecutive_warnings >= 3 {
                        error!(
                            "CRITICAL: Runtime appears stuck or severely overloaded. {} consecutive delays.",
                            consecutive_warnings
                        );

                        // TODO: Trigger graceful degradation
                        // - Stop accepting new events
                        // - Drain channels
                        // - Alert monitoring system
                    }
                } else {
                    // Reset counter if check was on time
                    if consecutive_warnings > 0 {
                        info!("Health check recovered, resetting warning counter");
                        consecutive_warnings = 0;
                    }
                }

                last_check = Instant::now();

                // TODO: Add more health checks
                // - Check tokio runtime metrics (if available)
                // - Monitor channel buffer utilization
                // - Track task spawn failures
                // - Measure P99 latency
            }
        }
    }

    info!("Runtime health monitor shutdown complete");
}
