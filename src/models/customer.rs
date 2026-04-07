use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Customer {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub industry: Option<String>,
    pub status: String,
    pub needs_review: bool,
    pub review_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Default)]
pub struct CreateCustomer {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub industry: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct UpdateCustomer {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub industry: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct StatusCount {
    pub status: String,
    pub count: i64,
}

pub const STATUSES: &[&str] = &["new", "contacted", "callback", "follow_up"];

impl Customer {
    pub async fn all(pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM customers ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
    }

    pub async fn find(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM customers WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM customers WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await
    }

    pub async fn search(pool: &PgPool, query: &str) -> Result<Vec<Self>, sqlx::Error> {
        let pattern = format!("%{}%", query);
        sqlx::query_as::<_, Self>(
            "SELECT * FROM customers 
             WHERE name ILIKE $1 OR email ILIKE $1 OR phone ILIKE $1 OR city ILIKE $1 OR industry ILIKE $1
             ORDER BY created_at DESC",
        )
        .bind(&pattern)
        .fetch_all(pool)
        .await
    }

    pub async fn filter_by_status(pool: &PgPool, status: &str) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM customers WHERE status = $1 ORDER BY created_at DESC")
            .bind(status)
            .fetch_all(pool)
            .await
    }

    pub async fn create(pool: &PgPool, data: &CreateCustomer) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "INSERT INTO customers (name, email, phone, website, city, state, industry, status, needs_review, review_reason) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, 'new', false, NULL) RETURNING *",
        )
        .bind(&data.name)
        .bind(&data.email)
        .bind(&data.phone)
        .bind(&data.website)
        .bind(&data.city)
        .bind(&data.state)
        .bind(&data.industry)
        .fetch_one(pool)
        .await
    }

    pub async fn update(&self, pool: &PgPool, data: &UpdateCustomer) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "UPDATE customers SET 
                name = COALESCE($2, name),
                email = COALESCE($3, email),
                phone = COALESCE($4, phone),
                website = COALESCE($5, website),
                city = COALESCE($6, city),
                state = COALESCE($7, state),
                industry = COALESCE($8, industry),
                status = COALESCE($9, status),
                updated_at = NOW()
             WHERE id = $1 RETURNING *",
        )
        .bind(self.id)
        .bind(&data.name)
        .bind(&data.email)
        .bind(&data.phone)
        .bind(&data.website)
        .bind(&data.city)
        .bind(&data.state)
        .bind(&data.industry)
        .bind(&data.status)
        .fetch_one(pool)
        .await
    }

    pub async fn update_status(pool: &PgPool, id: Uuid, status: &str) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "UPDATE customers SET status = $2, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(status)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM customers WHERE id = $1")
            .bind(self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn count_by_status(pool: &PgPool) -> Result<Vec<StatusCount>, sqlx::Error> {
        sqlx::query_as::<_, StatusCount>(
            "SELECT status, COUNT(*) as count FROM customers GROUP BY status"
        )
        .fetch_all(pool)
        .await
    }

    pub async fn total_count(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM customers")
            .fetch_one(pool)
            .await?;
        Ok(result.0)
    }

    pub async fn clear_review_flag(&self, pool: &PgPool) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "UPDATE customers SET needs_review = false, review_reason = NULL, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(self.id)
        .fetch_one(pool)
        .await
    }

    pub fn get_status_label(&self) -> &'static str {
        match self.status.as_str() {
            "new" => "New",
            "contacted" => "Contacted",
            "callback" => "Callback",
            "follow_up" => "Follow Up",
            _ => "Unknown",
        }
    }
}
