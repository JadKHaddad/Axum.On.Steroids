use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use anyhow::Context;
use the_axum::{
    error::ErrorVerbosity,
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

    tracing::info!("Starting ...");

    let socket_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);
    let error_verbosity = ErrorVerbosity::Full;
    let api_key_header_name = "x-api-key".to_string();
    let api_keys = vec!["api-key-1".to_string()];

    let server_config = ServerConfig::new(
        socket_address,
        error_verbosity,
        api_key_header_name,
        api_keys,
    );
    let server = Server::new(server_config);

    server.run().await?;

    Ok(())
}
