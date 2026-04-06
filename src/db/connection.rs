use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use tracing::info;

#[derive(Clone)]
pub struct DbConnection {
    pool: PgPool,
}

impl DbConnection {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    pub fn get(&self) -> &PgPool {
        &self.pool
    }
}

pub async fn establish_connection() -> PgPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Try to connect to the database, if it fails, create it
    match PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            info!("Database connection established");
            pool
        }
        Err(_) => {
            // Extract database name from URL
            let url = database_url.as_str();
            let parts: Vec<&str> = url.split('/').collect();
            let db_name = parts.last().unwrap_or(&"idea_share");
            let admin_url = format!("{}/{}", parts[0..parts.len()-1].join("/"), "postgres");

            info!("Database {} does not exist, creating...", db_name);
            info!("Using admin URL: {}", admin_url);

            // Connect to postgres database to create the new database
            let admin_pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(&admin_url)
                .await
                .expect("Failed to connect to postgres database");

            // Create the database
            sqlx::query(&format!("CREATE DATABASE {}", db_name))
                .execute(&admin_pool)
                .await
                .expect("Failed to create database");

            info!("Database {} created successfully", db_name);

            // Connect to the newly created database
            let pool = PgPoolOptions::new()
                .max_connections(10)
                .connect(&database_url)
                .await
                .expect("Failed to connect to newly created database");

            pool
        }
    }
}
