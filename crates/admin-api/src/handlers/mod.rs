pub mod applications;
pub mod endpoints;
// pub mod events; // TODO: Rewrite for ClickHouse (currently uses PostgreSQL syntax + missing tables)
pub mod events_stub;
pub use events_stub as events;
pub mod sse;
// pub mod statistics; // TODO: Rewrite for ClickHouse (currently uses PostgreSQL syntax + missing tables)
pub mod statistics_stub;
pub use statistics_stub as statistics;
pub mod users;
// pub mod websocket; // Deprecated - replaced with SSE (handlers/sse.rs)
