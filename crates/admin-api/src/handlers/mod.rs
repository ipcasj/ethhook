pub mod applications;
pub mod endpoints;
pub mod events; // ClickHouse-based implementation
pub mod sse;
pub mod statistics; // ClickHouse-based implementation
pub mod users;
// Stubs kept for reference but not used
// pub mod events_stub;
// pub mod statistics_stub;
// pub mod websocket; // Deprecated - replaced with SSE (handlers/sse.rs)
