use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> Result<PgPool, Box<dyn std::error::Error>> {
    let pool = PgPool::connect(database_url).await?;
    Ok(pool)
}
