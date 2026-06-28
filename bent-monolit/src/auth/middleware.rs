// use axum::{
//     body::Body,
//     extract::State,
//     http::{Request, StatusCode},
//     middleware::Next,
//     response::Response,
// };
// use tracing::info;

// use crate::auth::jwt::{Claims, verify_access_token};

// pub async fn auth_middleware(
//     State(jwt_secret): State<String>,
//     mut request: Request<Body>,
//     next: Next,
// ) -> Result<Response, StatusCode> {
//     let token = request
//         .headers()
//         .get("authorization")
//         .and_then(|v| v.to_str().ok())
//         .and_then(|v| v.strip_prefix("Bearer "))
//         .ok_or(StatusCode::UNAUTHORIZED)?;

//     let claims =
//         verify_access_token(token, jwt_secret.as_bytes()).map_err(|_| StatusCode::UNAUTHORIZED)?;

//     request.headers_mut().insert(
//         "x-user-id",
//         claims
//             .sub
//             .parse()
//             .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
//     );

//     let path = request.uri().path().to_string();
//     let method = request.method().to_string();

//     // Кладём claims в extensions — логгер достанет user_id отсюда
//     request.extensions_mut().insert(claims);
//     request.extensions_mut().insert(path);
//     request.extensions_mut().insert(method);

//     Ok(next.run(request).await)
// }

// pub async fn logger_middleware(request: Request<Body>, next: Next) -> Response {
//     let path = request.uri().path().to_string();
//     let method = request.method().to_string();

//     let user_id = request
//         .extensions()
//         .get::<Claims>()
//         .map(|s| s.sub.clone())
//         .unwrap_or("anonymous".to_string());

//     let start = std::time::Instant::now();
//     let response = next.run(request).await;
//     let duration_ms = start.elapsed().as_millis();

//     info!(
//         user_id = %user_id,
//         method = %method,
//         path = %path,
//         status = %response.status().as_u16(),
//         duration_ms = %duration_ms,
//         "incoming request"
//     );

//     response
// }
use crate::AppState;
use crate::auth::jwt::{Claims, verify_access_token};
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::info;

/// ВАЖНО: было State<String> в твоём исходнике — но в axum middleware
/// State достаётся из того же AppState, что висит на всём Router.
/// Раз jwt_secret теперь живёт внутри AppState.jwt, берём оттуда,
/// а не как отдельный extractor. Если предпочитаешь держать secret
/// отдельным State<String> — можно вернуть, но тогда придётся либо
/// делать два отдельных Router::with_state (неудобно), либо
/// доставать secret через FromRef<AppState> for String (коллизия,
/// если у тебя несколько String-полей в стейте). Проще — так.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = verify_access_token(token, state.jwt_secret.as_bytes())
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.headers_mut().insert(
        "x-user-id",
        claims
            .sub
            .parse()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    );

    let path = request.uri().path().to_string();
    let method = request.method().to_string();

    // Кладём claims в extensions — логгер достанет user_id отсюда
    request.extensions_mut().insert(claims);
    request.extensions_mut().insert(path);
    request.extensions_mut().insert(method);

    Ok(next.run(request).await)
}

pub async fn logger_middleware(request: Request<Body>, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().to_string();
    let user_id = request
        .extensions()
        .get::<Claims>()
        .map(|s| s.sub.clone())
        .unwrap_or("anonymous".to_string());

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
