use axum::{serve, middleware::from_fn, body::Body, extract::State, http::Request, middleware::Next, response::Response, Router};
use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, services::ServeDir};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod routes;
mod handlers;
mod auth;
mod models;
mod cache;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Establish database connection
    let pool = db::connection::establish_connection().await;

    // Initialize database tables
    db::init::init_database(&pool).await;

    // Seed database with initial data
    db::seed::seed_database(&pool).await;

    // Create session store
    let session_store = Arc::new(auth::session::SessionStore::new());

    // Create cache manager
    let cache_manager = Arc::new(cache::manager::CacheManager::new());
    cache_manager.init().expect("Failed to initialize cache manager");

    // Create database connection wrapper
    let db = db::connection::DbConnection::new(pool.clone());

    // Create app state
    let app_state = handlers::AppState {
        pool,
        db,
        session_store,
        cache_manager,
    };

    // Build routes
    let app = Router::new()
        // Serve static files from the uploads directory
        .nest_service("/uploads", ServeDir::new("uploads"))
        // Mount the application routes
        .merge(routes::create_router())
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .with_state(app_state);

    // Get host and port from environment
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).await.unwrap();

    // Start server
    println!("Server running at http://{}", addr);
    serve(listener, app)
        .await
        .unwrap();
}

