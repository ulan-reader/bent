// use crate::AppState;
// use crate::error::AppError;
// use crate::models::{Claims, Role};
// use axum::{extract::FromRequestParts, http::request::Parts};

// impl FromRequestParts<AppState> for Claims {
//     type Rejection = AppError;

//     async fn from_request_parts(
//         parts: &mut Parts,
//         state: &AppState,
//     ) -> Result<Self, Self::Rejection> {
//         let header = parts
//             .headers
//             .get(http::header::AUTHORIZATION)
//             .and_then(|v| v.to_str().ok());

//         let token = extract_bearer(header)?;
//         state.jwt.verify(token)
//     }
// }

// /// Обёртка для хендлеров, которым нужна конкретная роль —
// /// используй вместо ручной проверки claims.role внутри каждого хендлера.
// pub struct RequireAdmin(pub Claims);

// impl FromRequestParts<AppState> for RequireAdmin {
//     type Rejection = AppError;

//     async fn from_request_parts(
//         parts: &mut Parts,
//         state: &AppState,
//     ) -> Result<Self, Self::Rejection> {
//         let claims = Claims::from_request_parts(parts, state).await?;
//         if claims.role != Role::Admin {
//             return Err(AppError::Forbidden);
//         }
//         Ok(RequireAdmin(claims))
//     }
// }

// pub struct RequireInspector(pub Claims);

// impl FromRequestParts<AppState> for RequireInspector {
//     type Rejection = AppError;

//     async fn from_request_parts(
//         parts: &mut Parts,
//         state: &AppState,
//     ) -> Result<Self, Self::Rejection> {
//         let claims = Claims::from_request_parts(parts, state).await?;
//         if claims.role != Role::Inspector {
//             return Err(AppError::Forbidden);
//         }
//         Ok(RequireInspector(claims))
//     }
// }

// /// Claims, если заголовок Authorization присутствует и валиден — иначе None.
// /// Если заголовок ЕСТЬ, но JWT невалиден/просрочен — возвращаем 401,
// /// а не тихо проваливаемся в employee-флоу с мусорным токеном.
// pub struct OptionalClaims(pub Option<Claims>);

// impl FromRequestParts<AppState> for OptionalClaims {
//     type Rejection = AppError;

//     async fn from_request_parts(
//         parts: &mut Parts,
//         state: &AppState,
//     ) -> Result<Self, Self::Rejection> {
//         let header = parts
//             .headers
//             .get(http::header::AUTHORIZATION)
//             .and_then(|v| v.to_str().ok());

//         match header {
//             None => Ok(OptionalClaims(None)),
//             Some(_) => {
//                 let claims = Claims::from_request_parts(parts, state).await?;
//                 Ok(OptionalClaims(Some(claims)))
//             }
//         }
//     }
// }

// fn extract_bearer(header_value: Option<&str>) -> Result<&str, AppError> {
//     header_value
//         .ok_or(AppError::Unauthorized)?
//         .strip_prefix("Bearer ")
//         .ok_or(AppError::Unauthorized)
// }

// use crate::error::AppError;
// use crate::models::{Claims, TokenRole};
// use axum::{Extension, extract::FromRequestParts, http::request::Parts};

// /// Достаёт Claims, положенные туда auth_middleware. Если middleware
// /// не был подключен на этот роут — здесь будет 500, а не 401, поэтому
// /// убедись, что роут обёрнут в `.layer(from_fn_with_state(... auth_middleware))`.
// pub struct RequireUser(pub Claims);

// impl<S: Send + Sync> FromRequestParts<S> for RequireUser {
//     type Rejection = AppError;

//     async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
//         let Extension(claims) = Extension::<Claims>::from_request_parts(parts, state)
//             .await
//             .map_err(|_| AppError::Unauthorized)?;

//         if claims.role != TokenRole::User {
//             return Err(AppError::Forbidden);
//         }
//         Ok(RequireUser(claims))
//     }
// }

// pub struct RequireInspector(pub Claims);

// impl<S: Send + Sync> FromRequestParts<S> for RequireInspector {
//     type Rejection = AppError;

//     async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
//         let Extension(claims) = Extension::<Claims>::from_request_parts(parts, state)
//             .await
//             .map_err(|_| AppError::Unauthorized)?;

//         if claims.role != TokenRole::Inspector {
//             return Err(AppError::Forbidden);
//         }
//         Ok(RequireInspector(claims))
//     }
// }

// /// Claims если есть (любая роль), либо None — для роутов, куда могут
// /// прийти и анонимные employee-запросы, и авторизованные inspector-запросы.
// pub struct OptionalClaims(pub Option<Claims>);

// impl<S: Send + Sync> FromRequestParts<S> for OptionalClaims {
//     type Rejection = AppError;

//     async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
//         match Extension::<Claims>::from_request_parts(parts, state).await {
//             Ok(Extension(claims)) => Ok(OptionalClaims(Some(claims))),
//             Err(_) => Ok(OptionalClaims(None)),
//         }
//     }
// }
use crate::AppState;
use crate::auth::jwt::Claims;
use crate::error::AppError;
use axum::{extract::FromRequestParts, http::request::Parts};
use uuid::Uuid;

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
        crate::auth::jwt::verify_access_token(token, state.jwt_secret.as_bytes())
            .map_err(|_| AppError::Unauthorized)
    }
}

/// Любой залогиненный пользователь — без проверки конкретной роли/права.
/// Используй когда достаточно знать "это кто-то с валидной сессией"
/// (например, GET /api/auth/me, POST /api/auth/logout).
pub struct RequireUser(pub Claims);

impl FromRequestParts<AppState> for RequireUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let claims = Claims::from_request_parts(parts, state).await?;
        Ok(RequireUser(claims))
    }
}

/// Проверяет, что у пользователя есть конкретный permission (через
/// user_roles -> role_permissions). Делает доп. SQL-запрос — это
/// дороже, чем просто декодировать JWT, но так права можно менять
/// без переавторизации пользователя (отозвал роль — следующий же
/// запрос получит 403, а не ждёт истечения access-токена).
///
/// Использование: добавь permission_code как константу и проверяй
/// вручную в хендлере, либо заведи отдельный тип-маркер на каждый
/// permission, если хочешь компилятор-уровня гарантию на роуты.
pub struct RequirePermission(pub Claims);

impl RequirePermission {
    pub async fn check(
        state: &AppState,
        claims: &Claims,
        permission_code: &str,
    ) -> Result<(), AppError> {
        let user_id: Uuid = claims.sub.parse().map_err(|_| AppError::Unauthorized)?;

        let has_permission = sqlx::query_scalar!(
            r#"
            select exists(
                select 1
                from user_roles ur
                join role_permissions rp on rp.role_id = ur.role_id
                join permissions p on p.id = rp.permission_id
                where ur.user_id = $1 and p.code = $2
            )
            "#,
            user_id,
            permission_code,
        )
        .fetch_one(&state.pool)
        .await?
        .unwrap_or(false);

        if !has_permission {
            return Err(AppError::Forbidden);
        }

        Ok(())
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
