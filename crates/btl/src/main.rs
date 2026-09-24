use std::error::Error;
use dotenvy::dotenv;

use btl::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

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