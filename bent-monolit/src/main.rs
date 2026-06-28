// use axum::{
//     Router,
//     routing::{get, patch, post},
// };
// use http::Method;
// use sqlx::PgPool;
// use tower_http::cors::{Any, CorsLayer};

// mod auth;
// mod error;
// mod extractors;
// mod middleware;
// // твой существующий token-сервис (TokenService из исходного main.rs) можно
// // подключить без переписывания — просто добавь его как поле в AppState:
// //   pub tokens: TokenService,
// // а в хендлерах генерации/валидации токена бери State(state): State<AppState>
// // и вызывай state.tokens.generate(...) / state.tokens.validate(...).
// // Роуты для них верни в Router ниже.
// mod handlers;
// mod models;

// use auth::JwtService;

// #[derive(Clone)]
// pub struct AppState {
//     pub pool: PgPool,
//     pub jwt: JwtService,
// }

// #[tokio::main]
// async fn main() {
//     dotenvy::dotenv().ok();
//     tracing_subscriber::fmt::init();

//     let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is missing");
//     let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET is missing");

//     let pool = PgPool::connect(&database_url)
//         .await
//         .expect("Failed to connect to database");

//     let state = AppState {
//         pool,
//         jwt: JwtService::new(jwt_secret.as_bytes()),
//     };

//     let cors = CorsLayer::new()
//         .allow_origin(["http://localhost:5173".parse().unwrap()])
//         .allow_methods([
//             Method::GET,
//             Method::POST,
//             Method::PATCH, // было упущено в исходнике — фронт делает PATCH на update_status
//             Method::OPTIONS,
//         ])
//         .allow_headers(Any);

//     let app = Router::new()
//         .route("/api/submissions", post(handlers::create_submission))
//         .route("/api/admin/submissions", get(handlers::list_submissions))
//         .route(
//             "/api/admin/submissions/{id}/status",
//             patch(handlers::update_submission_status),
//         )
//         .route("/api/inspector/auth", post(handlers::inspector_auth))
//         .route("/api/admin/login", post(handlers::admin_login))
//         // .route("/api/token/generate", post(...)) — подключи свой существующий TokenService
//         // .route("/api/token/validate/{token}", get(...))
//         .with_state(state)
//         .layer(cors);

//     let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
//     println!("Listening on {}", addr);

//     let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
//     axum::serve(listener, app).await.unwrap();
// }
use axum::{
    Router, middleware as axum_middleware,
    routing::{get, patch, post},
};
use http::Method;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};

mod auth;
mod bot;
mod error;
mod extractors;
mod handlers;
mod models;

use auth::middleware::{auth_middleware, logger_middleware};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub public_base_url: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is missing");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET is missing");
    let public_base_url =
        std::env::var("PUBLIC_BASE_URL").unwrap_or_else(|_| "http://bent-control.kz".to_string());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let state = AppState {
        pool,
        jwt_secret,
        public_base_url: public_base_url.clone(),
    };

    let cors = CorsLayer::new()
        .allow_origin(["http://localhost:5173".parse().unwrap()])
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
        .allow_headers(Any);

    // Роуты, которым НЕ нужна авторизация (или у которых она опциональна
    // и проверяется вручную внутри хендлера — см. create_submission)
    let public_routes = Router::new()
        .route("/api/token/generate", post(handlers::generate_token))
        .route("/api/token/validate/{token}", get(handlers::validate_token))
        .route("/api/submissions", post(handlers::create_submission))
        .route("/api/inspector/auth", post(handlers::inspector_auth))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/refresh", post(handlers::refresh));

    // Роуты, которым нужен Bearer JWT — навешиваем auth_middleware слоем.
    // RequireUser/RequirePermission внутри хендлеров всё равно делают
    // финальную проверку, но auth_middleware даёт нам x-user-id в
    // заголовках и единый лог по запросу до того, как дойдёт до хендлера.
    let protected_routes = Router::new()
        .route("/api/auth/logout", post(handlers::logout))
        .route("/api/auth/logout_all", post(handlers::logout_all))
        .route("/api/auth/me", get(handlers::me))
        .route("/api/admin/submissions", get(handlers::list_submissions))
        .route(
            "/api/admin/submissions/{id}/status",
            patch(handlers::update_submission_status),
        )
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(axum_middleware::from_fn(logger_middleware))
        .with_state(state)
        .layer(cors);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // Сервер и бот живут в одном процессе, но это два независимых
    // event-loop'а — запускаем оба и ждём оба (на практике они оба
    // "вечные", так что join! просто держит процесс живым, пока
    // работают оба; если один упадёт — другой продолжит работать,
    // что обычно нежелательно для prod — лучше потом заменить на
    // select! с явным завершением процесса при падении любого из них).
    tokio::join!(
        async {
            axum::serve(listener, app).await.unwrap();
        },
        bot::run(),
    );
}
