use std::env;

pub struct Config {
    pub port: u16,
    pub cors_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let origins = env::var("CORS_ORIGIN")
            .unwrap_or_else(|_| "http://localhost:5173".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Self {
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            cors_origins: origins,
        }
    }
}
