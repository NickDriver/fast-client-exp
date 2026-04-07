use sqlx::PgPool;

const MIGRATIONS: &[(&str, &[&str])] = &[
    (
        "001_create_users",
        &[
            "CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                name TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
            "CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)",
        ],
    ),
    (
        "002_create_customers",
        &[
            "CREATE TABLE IF NOT EXISTS customers (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                name TEXT NOT NULL,
                email TEXT,
                phone TEXT,
                website TEXT,
                city TEXT,
                state TEXT,
                industry TEXT,
                status TEXT NOT NULL DEFAULT 'new',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
            "CREATE INDEX IF NOT EXISTS idx_customers_status ON customers(status)",
            "CREATE INDEX IF NOT EXISTS idx_customers_name ON customers(name)",
        ],
    ),
    (
        "003_create_customer_notes",
        &[
            "CREATE TABLE IF NOT EXISTS customer_notes (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
                body TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
            "CREATE INDEX IF NOT EXISTS idx_customer_notes_customer_id ON customer_notes(customer_id)",
        ],
    ),
];

pub async fn run(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            name TEXT PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    for (name, sqls) in MIGRATIONS {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE name = $1)",
        )
        .bind(name)
        .fetch_one(pool)
        .await?;

        if !exists {
            tracing::info!("Running migration: {}", name);
            for sql in *sqls {
                sqlx::query(sql).execute(pool).await?;
            }
            sqlx::query("INSERT INTO schema_migrations (name) VALUES ($1)")
                .bind(name)
                .execute(pool)
                .await?;
        }
    }

    Ok(())
}
