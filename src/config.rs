use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub api_key: String,
}

impl Config {
    pub fn from_env() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .expect("PORT must be a valid u16 number");
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://app.db?mode=rwc".to_string());
        let api_key = env::var("API_KEY")
            .unwrap_or_else(|_| "dev-secret-api-key".to_string());

        Self {
            host,
            port,
            database_url,
            api_key,
        }
    }
}
