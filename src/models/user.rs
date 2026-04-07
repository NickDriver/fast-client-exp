use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip)]
    pub password_hash: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
    pub name: String,
}

impl User {
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await
    }

    pub async fn create(pool: &PgPool, data: &CreateUser) -> Result<Self, sqlx::Error> {
        let password_hash = hash_password(&data.password).map_err(|e| sqlx::Error::Protocol(format!("Password hash error: {}", e).into()))?;
        let user = sqlx::query_as::<_, Self>(
            "INSERT INTO users (email, password_hash, name) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(&data.email)
        .bind(password_hash)
        .bind(&data.name)
        .fetch_one(pool)
        .await?;
        Ok(user)
    }

    pub fn verify_password(&self, password: &str) -> bool {
        use argon2::{Argon2, PasswordHash, password_hash::PasswordVerifier};
        let Ok(parsed_hash) = PasswordHash::new(&self.password_hash) else {
            return false;
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }
}

fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
    use rand::rngs::OsRng;
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}
