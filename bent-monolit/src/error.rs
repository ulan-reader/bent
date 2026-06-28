// use axum::{Json, http::StatusCode, response::IntoResponse};
// use serde::Serialize;
// use thiserror::Error;

// #[derive(Debug, Error)]
// pub enum AppError {
//     #[error("token not found")]
//     TokenNotFound,

//     #[error("token expired")]
//     TokenExpired,

//     #[error("token already used")]
//     TokenAlreadyUsed,

//     #[error("submission not found")]
//     SubmissionNotFound,

//     #[error("invalid tabel number")]
//     InvalidTabelNumber,

//     #[error("invalid credentials")]
//     InvalidCredentials,

//     #[error("missing or invalid authorization header")]
//     Unauthorized,

//     #[error("insufficient permissions")]
//     Forbidden,

//     #[error("invalid request: {0}")]
//     BadRequest(String),

//     #[error(transparent)]
//     Sqlx(#[from] sqlx::Error),

//     #[error(transparent)]
//     Jwt(#[from] jsonwebtoken::errors::Error),

//     #[error("password hashing error")]
//     PasswordHash,
// }

// #[derive(Serialize)]
// struct ErrorBody {
//     error: String,
// }

// impl IntoResponse for AppError {
//     fn into_response(self) -> axum::response::Response {
//         let status = match &self {
//             AppError::TokenNotFound | AppError::SubmissionNotFound => StatusCode::NOT_FOUND,
//             AppError::TokenExpired
//             | AppError::TokenAlreadyUsed
//             | AppError::InvalidTabelNumber
//             | AppError::InvalidCredentials
//             | AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
//             AppError::Unauthorized => StatusCode::UNAUTHORIZED,
//             AppError::Forbidden => StatusCode::FORBIDDEN,
//             AppError::Sqlx(_) | AppError::Jwt(_) | AppError::PasswordHash => {
//                 // Логируем подробности на сервере, наружу — generic-сообщение
//                 tracing::error!(error = ?self, "internal error");
//                 StatusCode::INTERNAL_SERVER_ERROR
//             }
//         };

//         let message = match status {
//             StatusCode::INTERNAL_SERVER_ERROR => "internal server error".to_string(),
//             _ => self.to_string(),
//         };

//         (status, Json(ErrorBody { error: message })).into_response()
//     }
// }
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("token not found")]
    TokenNotFound,

    #[error("token expired")]
    TokenExpired,

    #[error("token already used")]
    TokenAlreadyUsed,

    #[error("submission not found")]
    SubmissionNotFound,

    #[error("invalid tabel number")]
    InvalidTabelNumber,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("missing or invalid authorization header")]
    Unauthorized,

    #[error("insufficient permissions")]
    Forbidden,

    #[error("invalid request: {0}")]
    BadRequest(String),

    #[error("inspector not found or inactive")]
    InspectorNotFound,

    #[error("user not found")]
    UserNotFound,

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("jwt encode/decode error")]
    JwtCustom,

    #[error("password hashing error")]
    PasswordHash,
}

impl From<crate::auth::jwt::JwtError> for AppError {
    fn from(_: crate::auth::jwt::JwtError) -> Self {
        AppError::JwtCustom
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            AppError::TokenNotFound
            | AppError::SubmissionNotFound
            | AppError::InspectorNotFound
            | AppError::UserNotFound => StatusCode::NOT_FOUND,
            AppError::TokenExpired
            | AppError::TokenAlreadyUsed
            | AppError::InvalidTabelNumber
            | AppError::InvalidCredentials
            | AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::Sqlx(_) | AppError::Jwt(_) | AppError::JwtCustom | AppError::PasswordHash => {
                // Логируем подробности на сервере, наружу — generic-сообщение
                tracing::error!(error = ?self, "internal error");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let message = match status {
            StatusCode::INTERNAL_SERVER_ERROR => "internal server error".to_string(),
            _ => self.to_string(),
        };

        (status, Json(ErrorBody { error: message })).into_response()
    }
}
