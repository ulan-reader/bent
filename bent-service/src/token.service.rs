use chrono::{DateTime, Utc};

pub struct OneTimeToken {
    pub token: String,
    pub telegram_user_id: i64,
    pub used: bool,
    pub expires_at: DateTime<Utc>,
}

use chrono::{Duration, Utc};
use rand::{Rng, distr::Alphanumeric};
use sqlx::PgPool;

#[derive(Clone)]
pub struct TokenService {
    pool: PgPool,
}

impl TokenService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn generate(
        &self,
        telegram_user_id: i64,
    ) -> Result<String, TokenError> {
        let token: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
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

    pub async fn validate(
        &self,
        token: &str,
    ) -> Result<OneTimeToken, TokenError> {
        let row = sqlx::query_as!(
            OneTimeToken,
            r#"
            select
                token,
                telegram_user_id,
                used,
                expires_at
            from one_time_tokens
            where token = $1
            "#,
            token
        )
        .fetch_optional(&self.pool)
        .await?;

        let token = row.ok_or(TokenError::NotFound)?;

        if token.used {
            return Err(TokenError::AlreadyUsed);
        }

        if token.expires_at < Utc::now() {
            return Err(TokenError::Expired);
        }

        Ok(token)
    }

    pub async fn consume(
        &self,
        token: &str,
    ) -> Result<(), TokenError> {
        sqlx::query(
            r#"
            update one_time_tokens
            set used = true
            where token = $1
            "#,
        )
        .bind(token)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

use axum::{
    extract::State,
    Json,
};

pub async fn generate_token(
    State(service): State<TokenService>,
    Json(req): Json<GenerateTokenRequest>,
) -> Result<Json<GenerateTokenResponse>, TokenError> {
    let token = service
        .generate(req.telegram_user_id)
        .await?;

    Ok(Json(GenerateTokenResponse {
        url: format!(
            "https://your-domain.kz/form?token={}",
            token
        ),
        token,
    }))
}

use axum::{
    extract::{Path, State},
    Json,
};

pub async fn validate_token(
    State(service): State<TokenService>,
    Path(token): Path<String>,
) -> Result<Json<ValidateTokenResponse>, TokenError> {
    service.validate(&token).await?;

    Ok(Json(ValidateTokenResponse {
        valid: true,
    }))
}
