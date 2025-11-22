use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Application {
    #[cfg_attr(feature = "sqlx", sqlx(try_from = "String"))]
    pub id: Uuid,
    #[cfg_attr(feature = "sqlx", sqlx(try_from = "String"))]
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub webhook_secret: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct CreateApplicationRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    #[validate(length(max = 1000))]
    pub description: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateApplicationRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,

    #[validate(length(max = 1000))]
    pub description: Option<String>,

    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ApplicationResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub webhook_secret: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Application> for ApplicationResponse {
    fn from(app: Application) -> Self {
        Self {
            id: app.id,
            name: app.name,
            description: app.description,
            webhook_secret: app.webhook_secret,
            is_active: app.is_active,
            created_at: app.created_at,
            updated_at: app.updated_at,
        }
    }
}
