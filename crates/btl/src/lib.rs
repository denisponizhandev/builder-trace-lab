use std::error::Error;
use sqlx::postgres::{PgPoolOptions, PgPool};
use tokio::sync::mpsc;
use tokio::signal;

pub mod config;
pub mod pipeline;

use config::GlobalConfig;
use pipeline::message::PipelineMessage;
use pipeline::ingest::run_ingest;
use pipeline::processor::run_processor;

const CHANNEL_CAPACITY: usize = 16; 

pub struct App {
    name: String,
    config: GlobalConfig,
    pool: PgPool
}

impl App {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let global_config = GlobalConfig::from_env()?;
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&global_config.db_url)
            .await?;

        Ok(Self {
            name: String::from("builder-trace-lab"),
            config: global_config,
            pool
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error>>{

        println!("pipeline started");

        let (tx, rx) = mpsc::channel::<PipelineMessage>(CHANNEL_CAPACITY);

        let ingest_handle = tokio::spawn(run_ingest(tx));
        let processor_handle = tokio::spawn(run_processor(rx, self.pool.clone()));

        tokio::select! {
            _ = signal::ctrl_c() => {
                println!("shutdown requested");
                ingest_handle.abort();
            }
        }

        processor_handle.await??;

        match ingest_handle.await {
            Ok(Ok(())) => {
                println!("ingest stopped cleanly");
            }
            Ok(Err(e)) => {
                eprintln!("ingest returned error: {}", e);
            }
            Err(join_err) if join_err.is_cancelled() => {
                println!("ingest cancelled (expected on shutdown)");
            }
            Err(join_err) => {
                eprintln!("ingest task failed: {}", join_err);
            }
        }

        println!("shutdown complete");

        Ok(())
    }
}