use anyhow::Context;
use clap::Parser;
use the_axum::{
    cli_args::CliArgs,
    server::{Server, ServerConfig},
};

fn init_tracing() -> anyhow::Result<()> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt::Subscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .context("Failed to set global tracing subscriber")?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "server=trace,the_axum=trace,tower_http=trace");
    }

    init_tracing()?;

    let cli_args = CliArgs::parse();

    tracing::info!("Starting ...");

    let server_config = ServerConfig::from_config_file(cli_args.config_file).await?;
    let server = Server::new(server_config);

    server.run().await?;

    Ok(())
}
