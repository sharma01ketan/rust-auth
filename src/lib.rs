use std::sync::{Arc, Mutex};

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

pub mod identity;
pub mod tasks;

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

pub struct ManualClock {
    now: Mutex<DateTime<Utc>>,
}

impl ManualClock {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            now: Mutex::new(now),
        }
    }

    pub fn advance(&self, by: chrono::Duration) {
        *self.now.lock().expect("clock lock") += by;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> DateTime<Utc> {
        *self.now.lock().expect("clock lock")
    }
}

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub cache: Arc<Mutex<std::collections::HashMap<String, tasks::CachedMyTasks>>>,
    pub jwt_secret: String,
    pub code_pepper: String,
    pub clock: Arc<dyn Clock>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/seed/users", post(identity::seed_users))
        .route("/auth/login", post(identity::login))
        .route("/auth/verify-2fa", post(identity::verify))
        .route("/dev/email-logs/latest", get(identity::latest_email))
        .route("/tasks", post(tasks::create_task))
        .route("/tasks/assign", post(tasks::assign_tasks))
        .route("/tasks/view-my-tasks", get(tasks::view_my_tasks))
        .route("/tasks/{id}", patch(tasks::update_task))
        .with_state(state)
}

pub enum ApiError {
    Unauthorized,
    Forbidden,
    NotFound,
    BadRequest(&'static str),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".to_string()),
            Self::Forbidden => (StatusCode::FORBIDDEN, "forbidden".to_string()),
            Self::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message.to_string()),
            Self::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_string(),
            ),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        Self::Internal(error.to_string())
    }
}

pub fn stamp(now: DateTime<Utc>) -> String {
    now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

pub struct TestApp {
    pub router: Router,
    pub clock: Arc<ManualClock>,
}

impl TestApp {
    pub async fn new() -> Self {
        let clock = Arc::new(ManualClock::new(
            DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        ));
        let path = std::env::temp_dir().join(format!("task-api-{}.db", uuid::Uuid::new_v4()));
        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
            .unwrap()
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let state = AppState {
            pool,
            cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
            jwt_secret: "test-jwt-secret".to_string(),
            code_pepper: "test-pepper".to_string(),
            clock: clock.clone(),
        };
        Self {
            router: router(state),
            clock,
        }
    }
}
