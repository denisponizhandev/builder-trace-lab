use std::error::Error;
use sqlx::postgres::{PgPoolOptions, PgPool};

pub mod config;

use config::GlobalConfig;

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

        let one: i32 = sqlx::query_scalar("SELECT 1")
            .fetch_one(&self.pool)
            .await?;

        println!("db is connected, smoke ok: {}", one);
        println!("app {} has started!", &self.name);

        Ok(())
    }
}