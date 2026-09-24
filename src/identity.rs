use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use jsonwebtoken::{decode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{stamp, ApiError, AppState};

const SEED_USERS: &[(&str, &str, &str, &str)] = &[
    ("Admin", "admin@example.com", "admin-password", "admin"),
    (
        "James Bond",
        "jamesbond@example.com",
        "bond-password",
        "staff",
    ),
];

#[derive(Serialize, sqlx::FromRow)]
pub struct PublicUser {
    pub id: String,
    pub full_name: String,
    pub email: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct SeedResponse {
    users: Vec<PublicUser>,
}

pub async fn seed_users(State(state): State<AppState>) -> Result<Json<SeedResponse>, ApiError> {
    let now = stamp(state.clock.now());
    for (full_name, email, password, role) in SEED_USERS {
        let existing: Option<String> = sqlx::query_scalar("SELECT id FROM users WHERE email = ?1")
            .bind(email)
            .fetch_optional(&state.pool)
            .await?;
        if existing.is_some() {
            continue;
        }
        sqlx::query(
            "INSERT INTO users (id, full_name, email, hashed_password, role, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(full_name)
        .bind(email)
        .bind(hash_password(password)?)
        .bind(role)
        .bind(&now)
        .bind(&now)
        .execute(&state.pool)
        .await?;
    }

    let mut users = Vec::with_capacity(SEED_USERS.len());
    for (_, email, _, _) in SEED_USERS {
        users.push(
            sqlx::query_as::<_, PublicUser>(
                "SELECT id, full_name, email, role FROM users WHERE email = ?1",
            )
            .bind(email)
            .fetch_one(&state.pool)
            .await?,
        );
    }
    Ok(Json(SeedResponse { users }))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub login_challenge_id: String,
}

#[derive(sqlx::FromRow)]
struct PasswordUser {
    id: String,
    hashed_password: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let user =
        sqlx::query_as::<_, PasswordUser>("SELECT id, hashed_password FROM users WHERE email = ?1")
            .bind(&body.email)
            .fetch_optional(&state.pool)
            .await?;

    let Some(user) = user else {
        return Err(ApiError::Unauthorized);
    };
    if !password_matches(&body.password, &user.hashed_password)? {
        return Err(ApiError::Unauthorized);
    }

    let now = state.clock.now();
    let now_stamp = stamp(now);
    sqlx::query(
        "UPDATE login_challenges
         SET expires_at = ?1
         WHERE user_id = ?2 AND used_at IS NULL AND expires_at > ?1",
    )
    .bind(&now_stamp)
    .bind(&user.id)
    .execute(&state.pool)
    .await?;

    let code = format!("{:06}", Uuid::new_v4().as_u128() % 1_000_000);
    let challenge_id = Uuid::new_v4().to_string();
    let expires_at = stamp(now + chrono::Duration::minutes(5));
    sqlx::query(
        "INSERT INTO login_challenges (id, user_id, code_hash, expires_at, used_at, created_at)
         VALUES (?1, ?2, ?3, ?4, NULL, ?5)",
    )
    .bind(&challenge_id)
    .bind(&user.id)
    .bind(hash_code(&state.code_pepper, &code))
    .bind(&expires_at)
    .bind(&now_stamp)
    .execute(&state.pool)
    .await?;

    sqlx::query("INSERT INTO email_logs (id, to_email, code, created_at) VALUES (?1, ?2, ?3, ?4)")
        .bind(Uuid::new_v4().to_string())
        .bind(&body.email)
        .bind(&code)
        .bind(&now_stamp)
        .execute(&state.pool)
        .await?;

    Ok(Json(LoginResponse {
        login_challenge_id: challenge_id,
    }))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct EmailLogResponse {
    #[sqlx(rename = "to")]
    pub to: String,
    pub code: String,
    pub created_at: String,
}

pub async fn latest_email(
    State(state): State<AppState>,
) -> Result<Json<EmailLogResponse>, ApiError> {
    let log = sqlx::query_as::<_, EmailLogResponse>(
        "SELECT to_email AS \"to\", code, created_at
         FROM email_logs
         ORDER BY created_at DESC, rowid DESC
         LIMIT 1",
    )
    .fetch_optional(&state.pool)
    .await?;
    log.map(Json).ok_or(ApiError::NotFound)
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub login_challenge_id: String,
    pub code: String,
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub access_token: String,
}

#[derive(sqlx::FromRow)]
struct ChallengeRow {
    user_id: String,
    code_hash: String,
    expires_at: String,
    used_at: Option<String>,
}

#[derive(sqlx::FromRow)]
struct SessionUser {
    id: String,
    email: String,
    role: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: usize,
}

pub async fn verify(
    State(state): State<AppState>,
    Json(body): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, ApiError> {
    let challenge = sqlx::query_as::<_, ChallengeRow>(
        "SELECT user_id, code_hash, expires_at, used_at FROM login_challenges WHERE id = ?1",
    )
    .bind(&body.login_challenge_id)
    .fetch_optional(&state.pool)
    .await?;
    let Some(challenge) = challenge else {
        return Err(ApiError::Unauthorized);
    };

    let now = state.clock.now();
    let now_stamp = stamp(now);
    let code_ok = challenge.code_hash == hash_code(&state.code_pepper, &body.code);
    if challenge.used_at.is_some() || challenge.expires_at <= now_stamp || !code_ok {
        return Err(ApiError::Unauthorized);
    }

    sqlx::query("UPDATE login_challenges SET used_at = ?1 WHERE id = ?2")
        .bind(&now_stamp)
        .bind(&body.login_challenge_id)
        .execute(&state.pool)
        .await?;

    let user = sqlx::query_as::<_, SessionUser>("SELECT id, email, role FROM users WHERE id = ?1")
        .bind(&challenge.user_id)
        .fetch_one(&state.pool)
        .await?;
    let exp = (now + chrono::Duration::hours(8)).timestamp() as usize;
    let access_token = jsonwebtoken::encode(
        &Header::default(),
        &Claims {
            sub: user.id,
            email: user.email,
            role: user.role,
            exp,
        },
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|error| ApiError::Internal(error.to_string()))?;

    Ok(Json(VerifyResponse { access_token }))
}

pub fn current_user(state: &AppState, headers: &HeaderMap) -> Result<Claims, ApiError> {
    let header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    let Some(token) = header.strip_prefix("Bearer ") else {
        return Err(ApiError::Unauthorized);
    };
    if token.is_empty() {
        return Err(ApiError::Unauthorized);
    }
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = false;
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|_| ApiError::Unauthorized)?;
    if data.claims.exp <= state.clock.now().timestamp() as usize {
        return Err(ApiError::Unauthorized);
    }
    Ok(data.claims)
}

pub fn hash_code(pepper: &str, code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pepper.as_bytes());
    hasher.update(code.as_bytes());
    hex::encode(hasher.finalize())
}

fn password_matches(password: &str, hashed: &str) -> Result<bool, ApiError> {
    let parsed =
        PasswordHash::new(hashed).map_err(|error| ApiError::Internal(error.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

pub fn hash_password(password: &str) -> Result<String, ApiError> {
    let mut salt_bytes = [0u8; 16];
    getrandom::getrandom(&mut salt_bytes).map_err(|error| ApiError::Internal(error.to_string()))?;
    let salt = SaltString::encode_b64(&salt_bytes)
        .map_err(|error| ApiError::Internal(error.to_string()))?;
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| ApiError::Internal(error.to_string()))
}
