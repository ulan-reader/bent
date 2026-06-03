use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use chrono::{DateTime, Duration, Utc};
use http::Method;
use rand::RngExt;
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::net::SocketAddr;
use thiserror::Error;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is missing");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let token_service = TokenService::new(pool);

    let cors = CorsLayer::new()
        .allow_origin(["http://localhost:5173".parse().unwrap()])
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/token/generate", post(generate_token))
        .route("/api/token/validate/{token}", get(validate_token))
        .with_state(token_service)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

#[derive(Clone)]
pub struct TokenService {
    pool: PgPool,
}

impl TokenService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn generate(&self, telegram_user_id: i64) -> Result<String, TokenError> {
        let token: String = (0..64)
            .map(|_| rand::rng().sample(Alphanumeric) as char)
            .collect();

        let expires_at = Utc::now() + Duration::hours(24);

        sqlx::query(
            r#"
            insert into one_time_tokens (
                token,
                telegram_user_id,
                expires_at
            )
            values ($1, $2, $3)
            "#,
        )
        .bind(&token)
        .bind(telegram_user_id)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(token)
    }

    pub async fn validate(&self, token: &str) -> Result<(), TokenError> {
        let row = sqlx::query!(
            r#"
            select
                token,
                used,
                expires_at
            from one_time_tokens
            where token = $1
            "#,
            token
        )
        .fetch_optional(&self.pool)
        .await?;

        let row = row.ok_or(TokenError::NotFound)?;

        if row.used {
            return Err(TokenError::AlreadyUsed);
        }

        if row.expires_at < Utc::now() {
            return Err(TokenError::Expired);
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("token not found")]
    NotFound,

    #[error("token expired")]
    Expired,

    #[error("token already used")]
    AlreadyUsed,

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl axum::response::IntoResponse for TokenError {
    fn into_response(self) -> axum::response::Response {
        use axum::{http::StatusCode, response::IntoResponse};

        let status = match self {
            TokenError::NotFound => StatusCode::NOT_FOUND,
            TokenError::Expired => StatusCode::BAD_REQUEST,
            TokenError::AlreadyUsed => StatusCode::BAD_REQUEST,
            TokenError::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (
            status,
            Json(ErrorResponse {
                error: self.to_string(),
            }),
        )
            .into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
pub struct GenerateTokenRequest {
    pub telegram_user_id: i64,
}

#[derive(Serialize)]
pub struct GenerateTokenResponse {
    pub token: String,
    pub url: String,
}

#[derive(Serialize)]
pub struct ValidateTokenResponse {
    pub valid: bool,
}

async fn generate_token(
    State(service): State<TokenService>,
    Json(req): Json<GenerateTokenRequest>,
) -> Result<Json<GenerateTokenResponse>, TokenError> {
    let token = service.generate(req.telegram_user_id).await?;

    Ok(Json(GenerateTokenResponse {
        url: format!("http://localhost:5173/form?token={}", token),
        token,
    }))
}

async fn validate_token(
    State(service): State<TokenService>,
    Path(token): Path<String>,
) -> Result<Json<ValidateTokenResponse>, TokenError> {
    service.validate(&token).await?;

    Ok(Json(ValidateTokenResponse { valid: true }))
}
