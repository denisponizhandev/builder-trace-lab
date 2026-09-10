use std::error::Error;
use dotenvy::dotenv;

use btl::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    App::new().await?.start().await?;

    Ok(())
}
