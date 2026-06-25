use crate::auth::verify_access_token;
use crate::models::Claims;
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::info;

/// Проверяет Bearer JWT и кладёт Claims в extensions запроса.
/// Хендлеры достают их через `Extension<Claims>` или через кастомные
/// экстракторы RequireAdmin/RequireInspector (см. extractors.rs).
///
/// ВАЖНО: этот middleware НЕ проверяет revoked-статус сессии в БД —
/// он быстрый и stateless. Если нужна мгновенная инвалидация (logout_all),
/// добавь отдельную проверку sessions.revoked в самом хендлере или
/// сократи TTL access-токена и инвалидируй через истечение + refresh.
pub async fn auth_middleware(
    State(jwt_secret): State<String>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = verify_access_token(token, jwt_secret.as_bytes())
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

pub async fn logger_middleware(request: Request<Body>, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().to_string();
    let user_id = request
        .extensions()
        .get::<Claims>()
        .map(|c| c.sub.clone())
        .unwrap_or_else(|| "anonymous".to_string());

    let start = std::time::Instant::now();
    let response = next.run(request).await;
    let duration_ms = start.elapsed().as_millis();

    info!(
        user_id = %user_id,
        method = %method,
        path = %path,
        status = %response.status().as_u16(),
        duration_ms = %duration_ms,
        "incoming request"
    );

    response
}
