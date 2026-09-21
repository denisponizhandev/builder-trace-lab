use thiserror::Error;

#[derive(Debug, Error)]
pub enum GlobalConfigError {
    #[error("environment variable {0} is not set")]
    MissingEnv(&'static str)
}

pub struct GlobalConfig {
    pub db_url: String,
    pub http_bind: String
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
    
    pub fn from_env() -> Result<Self, GlobalConfigError> {
        let db_url = Self::db_url_from_env()?;
        let http_bind = Self::http_bind_from_env()?;

        Ok(GlobalConfig {
            db_url,
            http_bind
        })
    }
}