use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CustomerNote {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateCustomerNote {
    #[allow(dead_code)]
    pub body: String,
}

impl CustomerNote {
    pub async fn by_customer(pool: &PgPool, customer_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM customer_notes WHERE customer_id = $1 ORDER BY created_at DESC",
        )
        .bind(customer_id)
        .fetch_all(pool)
        .await
    }

    #[allow(dead_code)]
    pub async fn create(pool: &PgPool, customer_id: Uuid, data: &CreateCustomerNote) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "INSERT INTO customer_notes (customer_id, body) VALUES ($1, $2) RETURNING *",
        )
        .bind(customer_id)
        .bind(&data.body)
        .fetch_one(pool)
        .await
    }
}
