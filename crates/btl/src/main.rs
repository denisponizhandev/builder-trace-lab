use std::error::Error;
use dotenvy::dotenv;
use tracing_subscriber::EnvFilter;

use btl::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let app = match App::new().await {
        Ok(app) => app,
        Err(e) => {
            eprintln!("startup failed: {e}");
            std::process::exit(1);
        }
    };

    app.start().await?;

    Ok(())
}