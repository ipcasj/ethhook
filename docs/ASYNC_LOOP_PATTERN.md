# Why `loop {}` is Idiomatic in Async Rust

**`loop {}` in async Rust is NOT a thread** - it's an **event-driven async task** that yields cooperatively and consumes **zero CPU when idle**. This is the standard, idiomatic pattern for async servers in Rust 2025.

---

## The Confusion: Threads vs Async Tasks

### Traditional Thread Loop (DON'T DO THIS)

```rust
// ❌ BAD: This consumes a full CPU core even when idle!
std::thread::spawn(|| {
    loop {
        // This thread polls continuously, wasting CPU
        if let Some(work) = check_for_work() {
            do_work(work);
        }
        std::thread::sleep(Duration::from_millis(100)); // Still wastes resources
    }
});
```

**Problems:**

- 1 thread = 1 OS thread = ~8MB stack memory
- Consumes CPU even when idle (polling)
- Can't handle thousands of concurrent connections
- Hard to coordinate shutdown

### Async Task Loop (CORRECT ✅)

```rust
// ✅ GOOD: Event-driven, zero CPU when idle!
tokio::spawn(async {
    loop {
        tokio::select! {
            result = listener.accept() => {
                // Only runs when connection arrives
                handle_connection(result).await;
            }
            _ = shutdown_rx.recv() => {
                break; // Graceful exit
            }
        }
    }
});
```

**Benefits:**

- NOT a thread - runs on tokio's thread pool (shared)
- `tokio::select!` is **event-driven** - yields to other tasks when waiting
- Zero CPU usage when no events
- One task pool handles thousands of connections
- Cooperative multitasking

---

## How Async Loop Actually Works

### Behind the Scenes

```rust
loop {
    tokio::select! {
        result = listener.accept() => { /* ... */ }
        _ = shutdown.recv() => { break; }
    }
}
```

**What happens:**

1. **Event registration**: `tokio::select!` registers interest in:
   - TCP listener socket (epoll/kqueue/IOCP)
   - Broadcast channel
2. **Task suspension**: Task yields control to tokio runtime
3. **Other tasks run**: Tokio schedules other tasks on same thread
4. **Event arrives**: OS notifies tokio "socket has data!"
5. **Task wakes up**: Tokio resumes our task at exact `select!` point
6. **Process event**: Handle connection
7. **Repeat**: Loop back to step 1

**CPU usage**: Near zero when idle. Task only runs when events arrive.

---

## Real-World Comparison

### Metrics Server: Thread vs Async

#### Thread-Based (Old Way ❌)

```rust
// Spawns 1 OS thread per task
std::thread::spawn(|| {
    let listener = std::net::TcpListener::bind("0.0.0.0:9090")?;
    
    loop {
        // BLOCKS entire thread waiting for connection
        let (stream, _) = listener.accept()?;
        
        // Need another thread for each connection!
        std::thread::spawn(move || {
            handle_connection(stream);
        });
    }
});
```

**Resource usage at 1000 concurrent connections:**

- 1000+ OS threads
- ~8 GB RAM (8MB stack per thread)
- Context switching overhead kills performance

#### Async-Based (Modern Way ✅)

```rust
// Runs on tokio thread pool (typically 4-8 threads)
tokio::spawn(async {
    let listener = TcpListener::bind("0.0.0.0:9090").await?;
    
    loop {
        tokio::select! {
            // Yields to other tasks when no connections
            result = listener.accept() => {
                let (stream, _) = result?;
                
                // Spawn async task (NOT thread!)
                tokio::spawn(async move {
                    handle_connection(stream).await;
                });
            }
            _ = shutdown.recv() => { break; }
        }
    }
});
```

**Resource usage at 1000 concurrent connections:**

- 4-8 OS threads (tokio pool)
- ~50 MB RAM (futures are tiny)
- No context switching - cooperative scheduling

**Performance difference**: Async is **10-100x** more efficient!

---

## Our Production Implementation

### metrics.rs (Phase 7)

```rust
pub async fn start_metrics_server(
    port: u16,
    mut shutdown: tokio::sync::broadcast::Receiver<()>,
) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let semaphore = Arc::new(Semaphore::new(100)); // Bounded concurrency
    let mut tasks = JoinSet::new();

    loop {  // ✅ This is an async loop, NOT a thread!
        // Clean up completed tasks
        while tasks.try_join_next().is_some() {}

        tokio::select! {
            // Event-driven - only runs when connection arrives
            accept_result = listener.accept() => {
                if let Ok((stream, _)) = accept_result {
                    if let Ok(permit) = semaphore.clone().try_acquire_owned() {
                        tasks.spawn(async move {
                            let _permit = permit; // Released on drop
                            handle_connection(stream).await;
                        });
                    }
                }
            }
            
            // Graceful shutdown - event-driven
            _ = shutdown.recv() => {
                break; // Exit loop
            }
        }
    }
    
    // Wait for in-flight requests with timeout
    // ...
}
```

**Key features:**

1. **Event-driven**: `tokio::select!` yields when idle
2. **Bounded concurrency**: Semaphore limits to 100 connections
3. **Structured concurrency**: JoinSet tracks spawned tasks
4. **Graceful shutdown**: Shutdown signal breaks loop cleanly
5. **Resource cleanup**: Waits for in-flight requests before exit

---

## Other Rust Patterns (When to Use)

### 1. `std::thread::spawn` - Use for CPU-Intensive Work

```rust
// ✅ Good: CPU-bound computation
let handle = std::thread::spawn(|| {
    compute_expensive_hash(data)
});
```

**When to use:**

- CPU-intensive calculations (crypto, compression, encoding)
- Blocking system calls (file I/O on some platforms)
- Interfacing with C libraries that aren't thread-safe

### 2. `tokio::spawn` with `loop {}` - Use for I/O

```rust
// ✅ Good: I/O-bound async work
tokio::spawn(async {
    loop {
        tokio::select! {
            event = stream.next() => { /* ... */ }
            _ = shutdown.recv() => { break; }
        }
    }
});
```

**When to use:**

- Network servers (HTTP, WebSocket, TCP)
- Database connection pools
- Message queues
- Timers and intervals

### 3. `futures::stream::repeat_with` - Use for Lazy Sequences

```rust
// ✅ Good: Lazy infinite sequences
let events = futures::stream::repeat_with(|| {
    fetch_next_event()
}).take(100);
```

**When to use:**

- Generating infinite sequences lazily
- Polling external APIs on interval
- Retry logic

### 4. `tokio::time::interval` - Use for Periodic Tasks

```rust
// ✅ Good: Periodic health checks
let mut interval = tokio::time::interval(Duration::from_secs(30));
loop {
    interval.tick().await;
    perform_health_check().await;
}
```

**When to use:**

- Health checks
- Metrics collection
- Periodic cleanup tasks

---

## Key Improvements Made

### Before (Phase 5)

```rust
loop {
    // ❌ Polling shutdown signal (wastes CPU)
    if shutdown_rx.try_recv().is_ok() {
        break;
    }
    
    // Process events...
    
    // ❌ Fixed delay wastes time
    tokio::time::sleep(Duration::from_secs(5)).await;
}
```

**Problems:**

- `try_recv()` is polling (not event-driven)
- Fixed delays add latency
- No graceful shutdown coordination

### After (Phase 6 + 7)

```rust
loop {
    tokio::select! {
        // ✅ Event-driven - wakes only when connection arrives
        result = listener.accept() => { /* ... */ }
        
        // ✅ Event-driven shutdown - no polling!
        _ = shutdown.recv() => { 
            break; // Clean exit
        }
    }
}

// ✅ Graceful cleanup
while !tasks.is_empty() {
    tokio::select! {
        _ = tokio::time::sleep_until(deadline) => { break; }
        _ = tasks.join_next() => { /* wait for task */ }
    }
}
```

**Improvements:**

- Zero polling - pure event-driven
- Instant shutdown response
- Coordinated cleanup with timeout
- No wasted CPU cycles

---

## Summary

| Pattern | Use Case | CPU Usage | Concurrency Limit |
|---------|----------|-----------|-------------------|
| `std::thread` + `loop` | CPU-intensive work | High (constant) | ~1000 threads |
| `tokio::spawn` + `loop` + `select!` | I/O-bound async | Near zero idle | Millions of tasks |
| `futures::stream` | Lazy sequences | On-demand | Unlimited |
| `tokio::time::interval` | Periodic tasks | Near zero | Unlimited |

**Conclusion**: `loop {}` in async Rust is **idiomatic and correct**. It's not a thread - it's an event-driven async task that yields cooperatively. The combination of `loop {}` + `tokio::select!` is the **standard pattern** for async servers in Rust 2025.

---

## References

- [Tokio Tutorial - Select](https://tokio.rs/tokio/tutorial/select)
- [Async Book - Async vs Threads](https://rust-lang.github.io/async-book/01_getting_started/02_why_async.html)
- [Jon Gjengset - Async/Await in Depth](https://www.youtube.com/watch?v=ThjvMReOXYM)
