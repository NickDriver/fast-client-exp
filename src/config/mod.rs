use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct Settings {
    pub app_name: String,
    pub app_env: AppEnv,
    #[allow(dead_code)]
    pub app_debug: bool,
    #[allow(dead_code)]
    pub app_url: String,
    pub database_url: String,
    pub addr: SocketAddr,
}

#[derive(Clone, Debug)]
pub enum AppEnv {
    Development,
    Production,
}

impl AppEnv {
    #[allow(dead_code)]
    pub fn is_dev(&self) -> bool {
        matches!(self, Self::Development)
    }
}

impl Settings {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        let app_name = get_env("APP_NAME", "FastClient");
        let app_env = match get_env("APP_ENV", "development").as_str() {
            "production" => AppEnv::Production,
            _ => AppEnv::Development,
        };
        let app_debug = get_env("APP_DEBUG", "true").parse().unwrap_or(true);
        let app_url = get_env("APP_URL", "http://localhost:8000");

        // Build database URL from individual components or use DATABASE_URL if provided
        let database_url = if let Ok(url) = std::env::var("DATABASE_URL") {
            url
        } else {
            let host = get_env("DB_HOST", "localhost");
            let port = get_env("DB_PORT", "5432");
            let db = get_env("DB_DATABASE", "fastclient");
            let username = get_env("DB_USERNAME", "postgres");
            let password = get_env("DB_PASSWORD", "");

            if !password.is_empty() {
                format!(
                    "postgres://{}:{}@{}:{}/{}",
                    username, password, host, port, db
                )
            } else {
                format!("postgres://{}@{}/{}", username, host, db)
            }
        };

        let addr = get_env("APP_ADDR", "0.0.0.0:8000").parse()?;

        Ok(Self {
            app_name,
            app_env,
            app_debug,
            app_url,
            database_url,
            addr,
        })
    }
}

fn get_env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
