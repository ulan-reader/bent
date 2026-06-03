use serde::{Deserialize, Serialize};

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
