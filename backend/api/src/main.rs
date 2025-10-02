use intents_api::{Config, start_server};
use clap::Parser;
use eyre::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "intents-api")]
#[command(about = "Cross-Chain Orbital Intents AMM API Server")]
struct Cli {
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose {
        "debug,intents_api=trace,sqlx=info"
    } else {
        "info,intents_api=debug"
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    dotenv::dotenv().ok();
    let config = Config::from_file(&cli.config).await
        .or_else(|_| Config::from_env())
        .expect("Failed to load configuration");

    tracing::info!("Starting Intents API server with config: {:?}", config);

    // Start the server
    start_server(config).await?;

    Ok(())
}