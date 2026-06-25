use crate::AppState;
use crate::error::AppError;
use crate::models::{Claims, Role};
use axum::{extract::FromRequestParts, http::request::Parts};

impl FromRequestParts<AppState> for Claims {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        let token = extract_bearer(header)?;
        state.jwt.verify(token)
    }
}

/// Обёртка для хендлеров, которым нужна конкретная роль —
/// используй вместо ручной проверки claims.role внутри каждого хендлера.
pub struct RequireAdmin(pub Claims);

impl FromRequestParts<AppState> for RequireAdmin {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let claims = Claims::from_request_parts(parts, state).await?;
        if claims.role != Role::Admin {
            return Err(AppError::Forbidden);
        }
        Ok(RequireAdmin(claims))
    }
}

pub struct RequireInspector(pub Claims);

impl FromRequestParts<AppState> for RequireInspector {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let claims = Claims::from_request_parts(parts, state).await?;
        if claims.role != Role::Inspector {
            return Err(AppError::Forbidden);
        }
        Ok(RequireInspector(claims))
    }
}

/// Claims, если заголовок Authorization присутствует и валиден — иначе None.
/// Если заголовок ЕСТЬ, но JWT невалиден/просрочен — возвращаем 401,
/// а не тихо проваливаемся в employee-флоу с мусорным токеном.
pub struct OptionalClaims(pub Option<Claims>);

impl FromRequestParts<AppState> for OptionalClaims {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        match header {
            None => Ok(OptionalClaims(None)),
            Some(_) => {
                let claims = Claims::from_request_parts(parts, state).await?;
                Ok(OptionalClaims(Some(claims)))
            }
        }
    }
}

fn extract_bearer(header_value: Option<&str>) -> Result<&str, AppError> {
    header_value
        .ok_or(AppError::Unauthorized)?
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)
}
