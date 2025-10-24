use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct DeliveryAttempt {
    pub id: Uuid,
    pub event_id: Uuid,
    pub endpoint_id: Uuid,
    pub attempt_number: i32,
    pub http_status_code: Option<i32>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub attempted_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i32>,
    pub success: Option<bool>,
    pub should_retry: bool,
    pub next_retry_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryJob {
    pub event_id: Uuid,
    pub endpoint_id: Uuid,
    pub endpoint_url: String,
    pub hmac_secret: String,
    pub payload: String,
    pub attempt_number: i32,
    pub max_retries: i32,
    pub timeout_seconds: i32,
}

#[derive(Debug, Serialize)]
pub struct DeliveryAttemptResponse {
    pub id: Uuid,
    pub event_id: Uuid,
    pub endpoint_id: Uuid,
    pub attempt_number: i32,
    pub http_status_code: Option<i32>,
    pub error_message: Option<String>,
    pub attempted_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i32>,
    pub success: Option<bool>,
}

impl From<DeliveryAttempt> for DeliveryAttemptResponse {
    fn from(attempt: DeliveryAttempt) -> Self {
        Self {
            id: attempt.id,
            event_id: attempt.event_id,
            endpoint_id: attempt.endpoint_id,
            attempt_number: attempt.attempt_number,
            http_status_code: attempt.http_status_code,
            error_message: attempt.error_message,
            attempted_at: attempt.attempted_at,
            completed_at: attempt.completed_at,
            duration_ms: attempt.duration_ms,
            success: attempt.success,
        }
    }
}
