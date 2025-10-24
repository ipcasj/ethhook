use crate::auth;
use gloo_net::http::{Request, Response};
use serde::{Deserialize, Serialize};

const API_BASE: &str = "http://localhost:8080";

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

// Register returns the same structure as login
pub type RegisterResponse = LoginResponse;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserInfo {
    pub id: String, // UUID serialized as string
    pub email: String,
    pub name: String, // Changed from username to match API
}

#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub error: String,
}

pub async fn login(email: String, password: String) -> Result<LoginResponse, String> {
    let request = LoginRequest { email, password };

    let response = Request::post(&format!("{API_BASE}/api/v1/auth/login"))
        .json(&request)
        .map_err(|e| format!("Failed to create request: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}

pub async fn register(
    email: String,
    name: String,
    password: String,
) -> Result<RegisterResponse, String> {
    let request = RegisterRequest {
        email,
        name,
        password,
    };

    let response = Request::post(&format!("{API_BASE}/api/v1/auth/register"))
        .json(&request)
        .map_err(|e| format!("Failed to create request: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}

pub async fn get_profile() -> Result<UserInfo, String> {
    let token = auth::get_token().ok_or("Not authenticated")?;

    let response = Request::get(&format!("{API_BASE}/api/v1/users/me"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}

async fn handle_response<T: for<'de> Deserialize<'de>>(response: Response) -> Result<T, String> {
    if response.ok() {
        response
            .json::<T>()
            .await
            .map_err(|e| format!("Failed to parse response: {e}"))
    } else {
        let error = response
            .json::<ApiError>()
            .await
            .map(|e| e.error)
            .unwrap_or_else(|_| format!("HTTP error: {}", response.status()));
        Err(error)
    }
}

// ============================================================================
// Application API
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Application {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub api_key: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ApplicationListResponse {
    pub applications: Vec<Application>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct CreateApplicationRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateApplicationRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

/// List all applications for the authenticated user
pub async fn list_applications() -> Result<ApplicationListResponse, String> {
    let token = auth::get_token().ok_or("Not authenticated")?;

    let response = Request::get(&format!("{API_BASE}/api/v1/applications"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}

/// Get a specific application
pub async fn get_application(app_id: &str) -> Result<Application, String> {
    let token = auth::get_token().ok_or("Not authenticated")?;

    let response = Request::get(&format!("{API_BASE}/api/v1/applications/{app_id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}

/// Create a new application
pub async fn create_application(
    name: String,
    description: Option<String>,
) -> Result<Application, String> {
    let token = auth::get_token().ok_or("Not authenticated")?;
    let request = CreateApplicationRequest { name, description };

    let response = Request::post(&format!("{API_BASE}/api/v1/applications"))
        .header("Authorization", &format!("Bearer {token}"))
        .json(&request)
        .map_err(|e| format!("Failed to create request: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}

/// Update an application
pub async fn update_application(
    app_id: &str,
    name: Option<String>,
    description: Option<String>,
    is_active: Option<bool>,
) -> Result<Application, String> {
    let token = auth::get_token().ok_or("Not authenticated")?;
    let request = UpdateApplicationRequest {
        name,
        description,
        is_active,
    };

    let response = Request::put(&format!("{API_BASE}/api/v1/applications/{app_id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .json(&request)
        .map_err(|e| format!("Failed to create request: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}

/// Delete an application
pub async fn delete_application(app_id: &str) -> Result<(), String> {
    let token = auth::get_token().ok_or("Not authenticated")?;

    let response = Request::delete(&format!("{API_BASE}/api/v1/applications/{app_id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if response.ok() {
        Ok(())
    } else {
        let error = response
            .json::<ApiError>()
            .await
            .map(|e| e.error)
            .unwrap_or_else(|_| format!("HTTP error: {}", response.status()));
        Err(error)
    }
}

/// Regenerate API key for an application
pub async fn regenerate_api_key(app_id: &str) -> Result<Application, String> {
    let token = auth::get_token().ok_or("Not authenticated")?;

    let response = Request::post(&format!(
        "{API_BASE}/api/v1/applications/{app_id}/regenerate-key"
    ))
    .header("Authorization", &format!("Bearer {token}"))
    .send()
    .await
    .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}

// ============ Endpoints API ============

/// Webhook endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub id: String,
    pub application_id: String,
    pub name: String,
    pub webhook_url: String,
    pub description: Option<String>,
    pub hmac_secret: String,
    pub chain_ids: Vec<i32>,
    pub contract_addresses: Vec<String>,
    pub event_signatures: Vec<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// List of endpoints response
#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointListResponse {
    pub endpoints: Vec<Endpoint>,
    pub total: i64,
}

/// Create endpoint request
#[derive(Debug, Serialize)]
pub struct CreateEndpointRequest {
    pub application_id: String,
    pub webhook_url: String,
    pub description: Option<String>,
    pub chain_ids: Vec<i32>,
    pub contract_addresses: Vec<String>,
    pub event_signatures: Vec<String>,
}

/// Update endpoint request
#[derive(Debug, Serialize)]
pub struct UpdateEndpointRequest {
    pub webhook_url: Option<String>,
    pub description: Option<String>,
    pub chain_ids: Option<Vec<i32>>,
    pub contract_addresses: Option<Vec<String>>,
    pub event_signatures: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

/// List all endpoints for an application
pub async fn list_endpoints(app_id: &str) -> Result<EndpointListResponse, String> {
    let token = auth::get_token().ok_or("Not authenticated")?;

    let response = Request::get(&format!(
        "{API_BASE}/api/v1/applications/{app_id}/endpoints"
    ))
    .header("Authorization", &format!("Bearer {token}"))
    .send()
    .await
    .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}

/// List all endpoints for the authenticated user across all applications
pub async fn list_all_user_endpoints() -> Result<EndpointListResponse, String> {
    let token = auth::get_token().ok_or("Not authenticated")?;

    let response = Request::get(&format!("{API_BASE}/api/v1/endpoints"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}

/// Get a specific endpoint
pub async fn get_endpoint(endpoint_id: &str) -> Result<Endpoint, String> {
    let token = auth::get_token().ok_or("Not authenticated")?;

    let response = Request::get(&format!("{API_BASE}/api/v1/endpoints/{endpoint_id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}

/// Create a new endpoint
pub async fn create_endpoint(
    application_id: String,
    webhook_url: String,
    description: Option<String>,
    chain_ids: Vec<i32>,
    contract_addresses: Vec<String>,
    event_signatures: Vec<String>,
) -> Result<Endpoint, String> {
    let token = auth::get_token().ok_or("Not authenticated")?;

    let payload = CreateEndpointRequest {
        application_id,
        webhook_url,
        description,
        chain_ids,
        contract_addresses,
        event_signatures,
    };

    let body_str =
        serde_json::to_string(&payload).map_err(|e| format!("Serialization error: {e}"))?;

    let response = Request::post(&format!("{API_BASE}/api/v1/endpoints"))
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .body(body_str)
        .map_err(|e| format!("Request error: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}

/// Update an endpoint
pub async fn update_endpoint(
    endpoint_id: &str,
    webhook_url: Option<String>,
    description: Option<String>,
    chain_ids: Option<Vec<i32>>,
    contract_addresses: Option<Vec<String>>,
    event_signatures: Option<Vec<String>>,
    is_active: Option<bool>,
) -> Result<Endpoint, String> {
    let token = auth::get_token().ok_or("Not authenticated")?;

    let payload = UpdateEndpointRequest {
        webhook_url,
        description,
        chain_ids,
        contract_addresses,
        event_signatures,
        is_active,
    };

    let body_str =
        serde_json::to_string(&payload).map_err(|e| format!("Serialization error: {e}"))?;

    let response = Request::put(&format!("{API_BASE}/api/v1/endpoints/{endpoint_id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .body(body_str)
        .map_err(|e| format!("Request error: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}

/// Delete an endpoint
pub async fn delete_endpoint(endpoint_id: &str) -> Result<(), String> {
    let token = auth::get_token().ok_or("Not authenticated")?;

    let response = Request::delete(&format!("{API_BASE}/api/v1/endpoints/{endpoint_id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if response.ok() {
        Ok(())
    } else {
        let error = response
            .json::<ApiError>()
            .await
            .map(|e| e.error)
            .unwrap_or_else(|_| format!("HTTP error: {}", response.status()));
        Err(error)
    }
}

/// Regenerate HMAC secret for an endpoint
pub async fn regenerate_hmac_secret(endpoint_id: &str) -> Result<Endpoint, String> {
    let token = auth::get_token().ok_or("Not authenticated")?;

    let response = Request::post(&format!(
        "{API_BASE}/api/v1/endpoints/{endpoint_id}/regenerate-secret"
    ))
    .header("Authorization", &format!("Bearer {token}"))
    .send()
    .await
    .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}

// ============ Events API ============

/// Event from blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub block_number: i64,
    pub block_hash: String,
    pub transaction_hash: String,
    pub log_index: i32,
    pub contract_address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub ingested_at: String,
    pub processed_at: Option<String>,
    pub delivery_count: Option<i64>,
    pub successful_deliveries: Option<i64>,
}

/// List of events response
#[derive(Debug, Serialize, Deserialize)]
pub struct EventListResponse {
    pub events: Vec<Event>,
    pub total: i64,
}

/// Delivery attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryAttempt {
    pub id: String,
    pub event_id: String,
    pub endpoint_id: String,
    pub endpoint_name: String,
    pub attempt_number: i32,
    pub http_status_code: Option<i32>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub attempted_at: String,
    pub completed_at: Option<String>,
    pub duration_ms: Option<i32>,
    pub success: Option<bool>,
    pub should_retry: Option<bool>,
    pub next_retry_at: Option<String>,
}

/// List of delivery attempts response
#[derive(Debug, Serialize, Deserialize)]
pub struct DeliveryAttemptListResponse {
    pub delivery_attempts: Vec<DeliveryAttempt>,
    pub total: i64,
}

/// List events for user's endpoints
pub async fn list_events(endpoint_id: Option<&str>) -> Result<EventListResponse, String> {
    let token = auth::get_token().ok_or("Not authenticated")?;

    let url = if let Some(ep_id) = endpoint_id {
        format!("{API_BASE}/api/v1/events?endpoint_id={ep_id}")
    } else {
        format!("{API_BASE}/api/v1/events")
    };

    let response = Request::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}

/// Get a specific event by ID
pub async fn get_event(event_id: &str) -> Result<Event, String> {
    let token = auth::get_token().ok_or("Not authenticated")?;

    let response = Request::get(&format!("{API_BASE}/api/v1/events/{event_id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}

/// List delivery attempts
pub async fn list_delivery_attempts(
    event_id: Option<&str>,
    endpoint_id: Option<&str>,
) -> Result<DeliveryAttemptListResponse, String> {
    let token = auth::get_token().ok_or("Not authenticated")?;

    let mut url = format!("{API_BASE}/api/v1/delivery-attempts");
    let mut params = vec![];

    if let Some(ev_id) = event_id {
        params.push(format!("event_id={ev_id}"));
    }
    if let Some(ep_id) = endpoint_id {
        params.push(format!("endpoint_id={ep_id}"));
    }

    if !params.is_empty() {
        url = format!("{}?{}", url, params.join("&"));
    }

    let response = Request::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}

// ============ Statistics API ============

/// Dashboard statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStatistics {
    pub events_today: i64,
    pub events_total: i64,
    pub success_rate: f64,
    pub avg_delivery_time_ms: Option<f64>,
    pub active_endpoints: i64,
    pub total_deliveries: i64,
    pub successful_deliveries: i64,
    pub failed_deliveries: i64,
}

/// Get dashboard statistics
pub async fn get_dashboard_statistics() -> Result<DashboardStatistics, String> {
    let token = auth::get_token().ok_or("Not authenticated")?;

    let response = Request::get(&format!("{API_BASE}/api/v1/statistics/dashboard"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    handle_response(response).await
}
