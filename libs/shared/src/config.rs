use std::env;

pub struct AppConfig {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub session_ttl_secs: i64,
    pub cookie_secure: bool,
    pub cookie_domain: Option<String>,
}

// impl AppConfig {
//     pub fn from_env() -> anyhow::Result<Self> {
//         Ok(Self {
//             database_url: env::var("DATABASE_URL")
//                 .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?,
//             server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
//             server_port: env::var("SERVER_PORT")
//                 .unwrap_or_else(|_| "8080".into())
//                 .parse()
//                 .map_err(|_| anyhow::anyhow!("SERVER_PORT must be a valid port number"))?,
//         })
//     }
// }

type ConfigError = Box<dyn std::error::Error + Send + Sync>;

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?,
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "4000".into())
                .parse()
                .map_err(|_| "SERVER_PORT must be a valid port number")?,
            session_ttl_secs: env::var("SESSION_TTL_SECS")
                .unwrap_or_else(|_| "2592000".into()) // 30 ngày
                .parse()
                .map_err(|_| "SESSION_TTL_SECS must be an integer (seconds)")?,
            cookie_secure: env::var("COOKIE_SECURE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            cookie_domain: env::var("COOKIE_DOMAIN").ok(),
        })
    }
}
