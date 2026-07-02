use std::env;

pub struct AppConfig {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub session_ttl_secs: i64,
    pub cookie_secure: bool,
    pub cookie_domain: Option<String>,
    pub redis_url: String,
    /// TTL của bản cache phiên trong Redis (giây). Ngắn hơn session TTL để giới hạn staleness.
    pub session_cache_ttl_secs: i64,
    /// Khoảng tối thiểu giữa 2 lần ghi `expires_at` xuống DB (giây) — throttle.
    pub session_db_sync_secs: i64,
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
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            session_cache_ttl_secs: env::var("SESSION_CACHE_TTL_SECS")
                .unwrap_or_else(|_| "3600".into())
                .parse()
                .map_err(|_| "SESSION_CACHE_TTL_SECS must be an integer (seconds)")?,
            session_db_sync_secs: env::var("SESSION_DB_SYNC_SECS")
                .unwrap_or_else(|_| "60".into())
                .parse()
                .map_err(|_| "SESSION_DB_SYNC_SECS must be an integer (seconds)")?,
        })
    }
}
