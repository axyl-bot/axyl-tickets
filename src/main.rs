mod bot;
mod commands;
mod config;
mod logging;

use bot::run;
use config::Config;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    if let Err(e) = run_bot().await {
        eprintln!("Error: {}", e);
    }
}

async fn run_bot() -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(Config::new().await?);
    run(config).await?;
    Ok(())
}
