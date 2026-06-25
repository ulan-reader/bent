use axum::{
    Router,
    routing::{get, patch, post},
};
use http::Method;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};

mod auth;
mod error;
mod extractors;
// твой существующий token-сервис (TokenService из исходного main.rs) можно
// подключить без переписывания — просто добавь его как поле в AppState:
//   pub tokens: TokenService,
// а в хендлерах генерации/валидации токена бери State(state): State<AppState>
// и вызывай state.tokens.generate(...) / state.tokens.validate(...).
// Роуты для них верни в Router ниже.
mod handlers;
mod models;

use auth::JwtService;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt: JwtService,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is missing");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET is missing");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let state = AppState {
        pool,
        jwt: JwtService::new(jwt_secret.as_bytes()),
    };

    let cors = CorsLayer::new()
        .allow_origin(["http://localhost:5173".parse().unwrap()])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH, // было упущено в исходнике — фронт делает PATCH на update_status
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/submissions", post(handlers::create_submission))
        .route("/api/admin/submissions", get(handlers::list_submissions))
        .route(
            "/api/admin/submissions/{id}/status",
            patch(handlers::update_submission_status),
        )
        .route("/api/inspector/auth", post(handlers::inspector_auth))
        .route("/api/admin/login", post(handlers::admin_login))
        // .route("/api/token/generate", post(...)) — подключи свой существующий TokenService
        // .route("/api/token/validate/{token}", get(...))
        .with_state(state)
        .layer(cors);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
