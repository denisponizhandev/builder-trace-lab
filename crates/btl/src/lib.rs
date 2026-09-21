use std::error::Error;
use sqlx::postgres::{PgPoolOptions, PgPool};
use tokio::sync::mpsc;
use tokio::signal;

use axum;

pub mod config;
pub mod pipeline;
pub mod http;
pub mod storage;
pub mod domain;

use config::GlobalConfig;

use http::router::build_router;
use http::state::HttpState;

use pipeline::message::PipelineMessage;
use pipeline::processor::Processor;
use storage::bundles::BundleRepository;

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

        let state = HttpState::new(tx);
        let router = build_router(state);
        let listener = tokio::net::TcpListener::bind(&self.config.http_bind).await?;
        
        let http_handle = tokio::spawn(async move {
            axum::serve(listener, router).await
        });

        let bundle_repo = BundleRepository::new(self.pool.clone());
        let processor = Processor::new(bundle_repo);

        let simulation_is_on = self.config.simulation_is_on;
        let sleep_delay_ms = self.config.sleep_delay_ms;

        let processor_handle = tokio::spawn(async move {
            processor
                .run_processor(rx, simulation_is_on, sleep_delay_ms)
                .await
        });

        tokio::select! {
            _ = signal::ctrl_c() => {
                println!("shutdown requested");
                http_handle.abort();
            }
        }

        processor_handle.await??;

        match http_handle.await {
            Ok(Ok(())) => {
                println!("http stopped cleanly");
            }
            Ok(Err(e)) => {
                eprintln!("http returned error: {}", e);
            }
            Err(join_err) if join_err.is_cancelled() => {
                println!("http cancelled (expected on shutdown)");
            }
            Err(join_err) => {
                eprintln!("http task failed: {}", join_err);
            }
        }

        println!("shutdown complete");

        Ok(())
    }
}