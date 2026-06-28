use std::env;

pub struct AppConfig {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
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
        })
    }
}
