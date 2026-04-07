use sqlx::PgPool;

pub mod migrations;
pub mod pool;

pub use pool::create_pool;

pub async fn init_pool(database_url: &str) -> Result<PgPool, Box<dyn std::error::Error>> {
    let pool = create_pool(database_url).await?;
    migrations::run(&pool).await?;
    Ok(pool)
}
