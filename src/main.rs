use std::str::FromStr;
use std::sync::{Arc, Mutex};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use task_api::{router, AppState, SystemClock};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();
    let database_url = std::env::var("DATABASE_URL")?;
    let jwt_secret = std::env::var("JWT_SECRET")?;
    let code_pepper = std::env::var("CODE_PEPPER")?;
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    let options = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = AppState {
        pool,
        cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
        jwt_secret,
        code_pepper,
        clock: Arc::new(SystemClock),
    };
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    println!("task-api listening on {port}");
    axum::serve(listener, router(state)).await?;
    Ok(())
}
