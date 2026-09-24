use thiserror::Error;

#[derive(Debug, Error)]
pub enum GlobalConfigError {
    #[error("environment variable {0} is not set")]
    MissingEnv(&'static str),

    #[error("environment variable {0} must be an integer (milliseconds)")]
    InvalidDelayMs(&'static str),

    #[error("admission policy must be in (unbounded|wait|reject)")]
    InvalidPolicy
}

#[derive(Clone, Copy, Debug)]
pub enum AdmissionPolicy {
    Unbounded,
    WaitWhenFull,
    RejectWhenFull
}

pub struct GlobalConfig {
    pub db_url: String,
    pub http_bind: String,
    pub sleep_delay_ms: u64,
    pub simulation_queue_capacity: usize,
    pub result_queue_capacity: usize,
    pub admission_policy: AdmissionPolicy
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

    fn sleep_delay_from_env() -> Result<u64, GlobalConfigError> {
        std::env::var("SIMULATION_DELAY_MS")
            .map_err(|_| GlobalConfigError::MissingEnv("SIMULATION_DELAY_MS"))?
            .parse()
            .map_err(|_| GlobalConfigError::InvalidDelayMs("SIMULATION_DELAY_MS"))
    }

    fn simulation_queue_capacity_from_env() -> Result<usize, GlobalConfigError> {
        std::env::var("SIMULATION_QUEUE_CAPACITY")
            .map_err(|_| GlobalConfigError::MissingEnv("SIMULATION_QUEUE_CAPACITY"))?
            .parse()
            .map_err(|_| GlobalConfigError::InvalidDelayMs("SIMULATION_QUEUE_CAPACITY"))   
    }

    fn result_queue_capacity_from_env() -> Result<usize, GlobalConfigError> {
        std::env::var("RESULT_QUEUE_CAPACITY")
            .map_err(|_| GlobalConfigError::MissingEnv("RESULT_QUEUE_CAPACITY"))?
            .parse()
            .map_err(|_| GlobalConfigError::InvalidDelayMs("RESULT_QUEUE_CAPACITY"))   
    }

    fn admission_policy_from_env() -> Result<AdmissionPolicy, GlobalConfigError> {
        let policy = std::env::var("ADMISSION_POLICY").map_err(|_| {
            GlobalConfigError::MissingEnv("ADMISSION_POLICY")
        })?;

        match policy.as_str() {
            "unbounded" => Ok(AdmissionPolicy::Unbounded),
            "wait" => Ok(AdmissionPolicy::WaitWhenFull),
            "reject" => Ok(AdmissionPolicy::RejectWhenFull),
            _ => Err(GlobalConfigError::InvalidPolicy)
        }
    }
    
    pub fn from_env() -> Result<Self, GlobalConfigError> {
        let db_url = Self::db_url_from_env()?;
        let http_bind = Self::http_bind_from_env()?;
        let sleep_delay_ms = Self::sleep_delay_from_env()?;
        let simulation_queue_capacity = Self::simulation_queue_capacity_from_env()?;
        let result_queue_capacity = Self::result_queue_capacity_from_env()?;
        let admission_policy = Self::admission_policy_from_env()?;
        
        Ok(GlobalConfig {
            db_url,
            http_bind,
            sleep_delay_ms,
            simulation_queue_capacity,
            result_queue_capacity,
            admission_policy
        })
    }
}