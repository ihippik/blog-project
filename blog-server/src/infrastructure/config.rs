use serde::Deserialize;

/// Application configuration.
///
/// Loaded from environment variables.
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    /// Server host address.
    pub host: String,

    /// HTTP server port.
    pub http_port: u16,

    /// gRPC server port.
    pub grpc_port: u16,

    /// Database connection URL.
    pub database_url: String,

    /// Secret key used for JWT signing.
    pub jwt_secret: String,

    /// Allowed CORS origins.
    ///
    /// Defaults to `*` if not specified.
    #[serde(default)]
    pub cors_origins: Vec<String>,

    /// Logging output format.
    pub log_format: String,
}

impl AppConfig {
    /// Loads configuration from environment variables.
    ///
    /// Uses `.env` file if present.
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let http_port = std::env::var("HTTP_PORT")
            .unwrap_or_else(|_| "8080".into())
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid HTTP PORT: {}", e))?;
        let grpc_port: u16 = std::env::var("GRPC_PORT")
            .unwrap_or_else(|_| "5000".into())
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid GRPC PORT: {}", e))?;
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;
        let jwt_secret =
            std::env::var("JWT_SECRET").map_err(|_| anyhow::anyhow!("JWT_SECRET must be set"))?;
        let cors_origins = std::env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "*".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "text".into());

        Ok(Self {
            host,
            http_port,
            grpc_port,
            database_url,
            jwt_secret,
            cors_origins,
            log_format,
        })
    }
}
