use thiserror::Error;

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
