use sqlx::postgres::PgPoolOptions;
use std::env;
use dotenv::dotenv;

pub async fn connect_to_database() -> sqlx::Pool<sqlx::Postgres> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool.");

    // println!("Connected to the database!");

    return pool;
}

pub async fn check_database_connection() -> bool {
    let pool = connect_to_database().await;

    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => true,
        Err(_) => false,
    }
}