mod auth;
mod config;
mod database;
mod handlers;
mod models;

use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};
use sqlx::PgPool;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "fastclient=debug,tower_http=debug,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let settings = config::Settings::from_env()?;
    tracing::info!("Starting {} in {:?} mode", settings.app_name, settings.app_env);

    let pool = database::init_pool(&settings.database_url).await?;
    tracing::info!("Database connected and migrations applied");

    let state = AppState { pool };

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(time::Duration::hours(24)));

    let auth_routes = Router::new()
        .route("/login", get(handlers::auth::login_page).post(handlers::auth::login))
        .route("/register", get(handlers::register::register_page).post(handlers::register::register))
        .route("/logout", get(handlers::auth::logout));

    let protected_routes = Router::new()
        .route("/", get(handlers::dashboard::dashboard))
        .route("/customers", get(handlers::customers::index).post(handlers::customers::create))
        .route("/customers/new", get(handlers::customers::create_page))
        .route("/customers/export", get(handlers::csv::export))
        .route("/customers/import", get(handlers::import::import_page).post(handlers::csv::import))
        .route("/customers/scrape", post(handlers::scrape::scrape))
        .route("/customers/{id}", get(handlers::customers::show))
        .route("/customers/{id}/edit", get(handlers::customers::edit_page))
        .route("/customers/{id}", put(handlers::customers::update))
        .route("/customers/{id}/status", post(handlers::customers::update_status))
        .route("/customers/{id}/delete", post(handlers::customers::delete))
        .route("/customers/{id}/notes", post(handlers::notes::create))
        .route("/customers/{id}/clear-review", post(handlers::customers::clear_review))
        .layer(middleware::from_fn(auth::require_auth));

    let app = Router::new()
        .merge(auth_routes)
        .merge(protected_routes)
        .nest_service("/assets", ServeDir::new("src/static"))
        .layer(middleware::from_fn(handlers::auth::inject_user_id))
        .layer(session_layer)
        .with_state(state);

    tracing::info!("Listening on {}", settings.addr);
    let listener = tokio::net::TcpListener::bind(&settings.addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
