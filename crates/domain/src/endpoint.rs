use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Endpoint {
    pub id: Uuid,
    pub application_id: Uuid,
    pub name: String,
    pub url: String,
    pub hmac_secret: String,
    pub contract_address: Option<String>,
    pub event_topics: Option<Vec<String>>,
    pub rate_limit_per_second: i32,
    pub max_retries: i32,
    pub timeout_seconds: i32,
    pub is_active: bool,
    pub health_status: HealthStatus,
    pub last_successful_delivery_at: Option<DateTime<Utc>>,
    pub consecutive_failures: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Failed,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateEndpointRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    
    #[validate(url)]
    pub url: String,
    
    #[validate(length(min = 42, max = 42))]
    pub contract_address: Option<String>,
    
    pub event_topics: Option<Vec<String>>,
    
    #[validate(range(min = 1, max = 100))]
    pub rate_limit_per_second: Option<i32>,
    
    #[validate(range(min = 0, max = 10))]
    pub max_retries: Option<i32>,
    
    #[validate(range(min = 5, max = 300))]
    pub timeout_seconds: Option<i32>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateEndpointRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    
    #[validate(url)]
    pub url: Option<String>,
    
    pub is_active: Option<bool>,
    
    #[validate(range(min = 1, max = 100))]
    pub rate_limit_per_second: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct EndpointResponse {
    pub id: Uuid,
    pub application_id: Uuid,
    pub name: String,
    pub url: String,
    pub hmac_secret: String,
    pub contract_address: Option<String>,
    pub event_topics: Option<Vec<String>>,
    pub rate_limit_per_second: i32,
    pub max_retries: i32,
    pub timeout_seconds: i32,
    pub is_active: bool,
    pub health_status: HealthStatus,
    pub last_successful_delivery_at: Option<DateTime<Utc>>,
    pub consecutive_failures: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Endpoint> for EndpointResponse {
    fn from(endpoint: Endpoint) -> Self {
        Self {
            id: endpoint.id,
            application_id: endpoint.application_id,
            name: endpoint.name,
            url: endpoint.url,
            hmac_secret: endpoint.hmac_secret,
            contract_address: endpoint.contract_address,
            event_topics: endpoint.event_topics,
            rate_limit_per_second: endpoint.rate_limit_per_second,
            max_retries: endpoint.max_retries,
            timeout_seconds: endpoint.timeout_seconds,
            is_active: endpoint.is_active,
            health_status: endpoint.health_status,
            last_successful_delivery_at: endpoint.last_successful_delivery_at,
            consecutive_failures: endpoint.consecutive_failures,
            created_at: endpoint.created_at,
            updated_at: endpoint.updated_at,
        }
    }
}
