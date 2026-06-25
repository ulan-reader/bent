use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub sid: String,
    pub exp: usize,
}

#[derive(Debug)]
pub enum JwtError {
    Encode,
    Decode,
}

pub fn create_access_token(
    user_id: Uuid,
    session_id: Uuid,
    secret: &[u8],
) -> Result<String, JwtError> {
    let claims = Claims {
        sub: user_id.to_string(),
        sid: session_id.to_string(),
        exp: (Utc::now() + Duration::minutes(15)).timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&secret),
    )
    .map_err(|_| JwtError::Encode)
}

pub fn verify_access_token(token: &str, secret: &[u8]) -> Result<Claims, JwtError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|_| JwtError::Decode)
}
