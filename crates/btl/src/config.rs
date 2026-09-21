use thiserror::Error;

#[derive(Debug, Error)]
pub enum GlobalConfigError {
    #[error("environment variable {0} is not set")]
    MissingEnv(&'static str),

    #[error("environment variable {0} must be an integer (milliseconds)")]
    InvalidDelayMs(&'static str),

    #[error("environment variable {0} must be a bool")]
    InvalidDelayOn(&'static str)
}

pub struct GlobalConfig {
    pub db_url: String,
    pub http_bind: String,
    pub sleep_delay_ms: u64,
    pub simulation_is_on: bool
}

impl GlobalConfig {
    fn db_url_from_env() -> Result<String, GlobalConfigError> {
        std::env::var("DATABASE_URL").map_err(|_| {
            GlobalConfigError::MissingEnv("DATABASE_URL")
        })
    }

    fn http_bind_from_env() -> Result<String, GlobalConfigError> {
        std::env::var("HTTP_BIND").map_err(|_| {
            GlobalConfigError::MissingEnv("HTTP_BIND")
        })
    }

    fn sleep_delay_is_on() -> Result<bool, GlobalConfigError> {
        std::env::var("SIMULATION_DELAY_ON")
            .map_err(|_| GlobalConfigError::MissingEnv("SIMULATION_DELAY_ON"))?
            .parse()
            .map_err(|_| GlobalConfigError::InvalidDelayOn("SIMULATION_DELAY_ON"))
    }

    fn sleep_delay_from_env() -> Result<u64, GlobalConfigError> {
        std::env::var("SIMULATION_DELAY_MS")
            .map_err(|_| GlobalConfigError::MissingEnv("SIMULATION_DELAY_MS"))?
            .parse()
            .map_err(|_| GlobalConfigError::InvalidDelayMs("SIMULATION_DELAY_MS"))
    }
    
    pub fn from_env() -> Result<Self, GlobalConfigError> {
        let db_url = Self::db_url_from_env()?;
        let http_bind = Self::http_bind_from_env()?;
        let sleep_delay_ms = Self::sleep_delay_from_env()?;
        let simulation_is_on = Self::sleep_delay_is_on()?;

        Ok(GlobalConfig {
            db_url,
            http_bind,
            sleep_delay_ms,
            simulation_is_on
        })
    }
}