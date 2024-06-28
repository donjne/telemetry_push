use std::error::Error;
use sqlx::postgres::PgPoolOptions;
use std::env;
use dotenvy::dotenv;

mod metrics;
mod server;
mod user;
mod error;
mod auth;
mod functionalities;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Get the database URL from the environment
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");
    println!("Listening on port 8080");
    server::run_server(pool).await;
    Ok(())
}
