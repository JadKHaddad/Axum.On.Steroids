use std::net::SocketAddr;

use anyhow::Context;
use axum::{middleware, routing::get, Router};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    decompression::RequestDecompressionLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};

use crate::{
    error::ErrorVerbosity, middleware::trace_response_body::trace_response_body, state::ApiState,
};

pub struct ServerConfig {
    socket_address: SocketAddr,
    error_verbosity: ErrorVerbosity,
}

impl ServerConfig {
    pub fn new(socket_address: SocketAddr, error_verbosity: ErrorVerbosity) -> Self {
        Self {
            socket_address,
            error_verbosity,
        }
    }
}

pub struct Server {
    config: ServerConfig,
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let state = ApiState::new(self.config.error_verbosity);

        let app = Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                trace_response_body,
            ))
            .with_state(state)
            .layer(
                ServiceBuilder::new()
                    .layer(
                        TraceLayer::new_for_http()
                            .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                            .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                            .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
                    )
                    .layer(RequestDecompressionLayer::new())
                    .layer(CompressionLayer::new())
                    .layer(CorsLayer::permissive()),
            );

        tracing::info!(addr = %self.config.socket_address, "Starting server");

        let listener = TcpListener::bind(&self.config.socket_address)
            .await
            .context("Bind failed")?;

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Server failed")?;

        Ok(())
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");

        tracing::info!("CTRL+C received");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM signal handler")
            .recv()
            .await;

        tracing::info!("SIGTERM received");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutting down");
}
