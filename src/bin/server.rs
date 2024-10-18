use rust_web_dev::{run, setup_store, Config};

#[tokio::main]
async fn main() -> Result<(), handle_errors::Error> {
    dotenvy::dotenv().ok();

    let config = Config::new().expect("Config can't not be set");
    let store = setup_store(&config).await?;

    tracing::info!("Q&A service build ID {}", env!("RUST_WEB_DEV_VERSION"));

    run(config, store).await;

    Ok(())
}
