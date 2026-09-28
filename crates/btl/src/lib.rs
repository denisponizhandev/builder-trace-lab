use std::error::Error;

use sqlx::postgres::{PgPoolOptions, PgPool};
use tokio::signal;

pub mod config;
pub mod pipeline;
pub mod http;
pub mod storage;
pub mod domain;
pub mod admission;
pub mod metrics;
mod runtime;

use config::GlobalConfig;
use runtime::{build_job_channels, build_metrics, spawn_stages, wait_shutdown};

pub struct App {
    config: GlobalConfig,
    pool: PgPool,
}

impl App {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let global_config = GlobalConfig::from_env()?;

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&global_config.db_url)
            .await?;

        Ok(Self {
            config: global_config,
            pool,
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error>> {
        println!("pipeline started");

        let policy = self.config.admission_policy;
        let channel_cap = self.config.simulation_queue_capacity;

        let channels = build_job_channels(policy, channel_cap);
        let metrics = build_metrics(policy, channel_cap);
        let handles = spawn_stages(&self.config, &self.pool, channels, metrics).await?;

        signal::ctrl_c().await?;
        wait_shutdown(handles).await;

        Ok(())
    }
}
