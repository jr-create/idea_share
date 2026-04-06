use sqlx::Pool;
use sqlx::postgres::Postgres;
use std::sync::Arc;
use crate::auth::session::SessionStore;
use crate::cache::manager::CacheManager;
use crate::db::connection::DbConnection;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub db: DbConnection,
    pub session_store: Arc<SessionStore>,
    pub cache_manager: Arc<CacheManager>,
}