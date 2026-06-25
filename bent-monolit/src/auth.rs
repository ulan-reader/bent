use crate::error::AppError;
use crate::models::{Claims, Role};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }

    /// ttl_hours: для inspector обычно короче (смена), для admin — длиннее
    pub fn issue(&self, sub: &str, role: Role, ttl_hours: i64) -> Result<String, AppError> {
        let now = Utc::now();
        let claims = Claims {
            sub: sub.to_string(),
            role,
            iat: now.timestamp(),
            exp: (now + Duration::hours(ttl_hours)).timestamp(),
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)?;
        Ok(token)
    }

    pub fn verify(&self, token: &str) -> Result<Claims, AppError> {
        let data = decode::<Claims>(token, &self.decoding_key, &Validation::default())?;
        Ok(data.claims)
    }
}
