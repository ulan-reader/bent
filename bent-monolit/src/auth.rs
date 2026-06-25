// use crate::error::AppError;
// use crate::models::{Claims, Role};
// use chrono::{Duration, Utc};
// use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

// #[derive(Clone)]
// pub struct JwtService {
//     encoding_key: EncodingKey,
//     decoding_key: DecodingKey,
// }

// impl JwtService {
//     pub fn new(secret: &[u8]) -> Self {
//         Self {
//             encoding_key: EncodingKey::from_secret(secret),
//             decoding_key: DecodingKey::from_secret(secret),
//         }
//     }

//     /// ttl_hours: для inspector обычно короче (смена), для admin — длиннее
//     pub fn issue(&self, sub: &str, role: Role, ttl_hours: i64) -> Result<String, AppError> {
//         let now = Utc::now();
//         let claims = Claims {
//             sub: sub.to_string(),
//             role,
//             iat: now.timestamp(),
//             exp: (now + Duration::hours(ttl_hours)).timestamp(),
//         };

//         let token = encode(&Header::default(), &claims, &self.encoding_key)?;
//         Ok(token)
//     }

//     pub fn verify(&self, token: &str) -> Result<Claims, AppError> {
//         let data = decode::<Claims>(token, &self.decoding_key, &Validation::default())?;
//         Ok(data.claims)
//     }
// }

use crate::models::{Claims, TokenRole};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use uuid::Uuid;

#[derive(Debug)]
pub enum JwtError {
    Encode,
    Decode,
}

/// Access-токен для обычного пользователя (RBAC), привязан к сессии в БД —
/// sid позволяет инвалидировать его через revoked=true без ожидания истечения exp.
pub fn create_user_access_token(
    user_id: Uuid,
    session_id: Uuid,
    secret: &[u8],
) -> Result<String, JwtError> {
    let claims = Claims {
        sub: user_id.to_string(),
        sid: session_id.to_string(),
        role: TokenRole::User,
        exp: (Utc::now() + Duration::minutes(15)).timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|_| JwtError::Encode)
}

/// Токен для инспектора — без записи в `sessions` (короткоживущий, на смену),
/// sid здесь просто случайный UUID для трассировки в логах, не для revoke.
pub fn create_inspector_access_token(tabel_number: i64, secret: &[u8]) -> Result<String, JwtError> {
    let claims = Claims {
        sub: tabel_number.to_string(),
        sid: Uuid::new_v4().to_string(),
        role: TokenRole::Inspector,
        exp: (Utc::now() + Duration::hours(12)).timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
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
