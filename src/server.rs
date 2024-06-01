use std::net::SocketAddr;

use anyhow::Context;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    decompression::RequestDecompressionLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};

use crate::{
    error::ErrorVerbosity,
    middleware::{
        method_not_allowed::method_not_allowed, trace_headers::trace_headers,
        trace_response_body::trace_response_body, validate_api_key_and_put_as_extension,
    },
    route::{
        api_key_protected, extract_api_key, extract_valid_api_key, extract_valid_api_key_optional,
        post_json,
    },
    state::ApiState,
};

pub struct ServerConfig {
    socket_address: SocketAddr,
    error_verbosity: ErrorVerbosity,
    api_key_header_name: String,
    api_keys: Vec<String>,
}

impl ServerConfig {
    pub fn new(
        socket_address: SocketAddr,
        error_verbosity: ErrorVerbosity,
        api_key_header_name: String,
        api_keys: Vec<String>,
    ) -> Self {
        Self {
            socket_address,
            error_verbosity,
            api_key_header_name,
            api_keys,
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
        let state = ApiState::new(
            self.config.error_verbosity,
            self.config.api_key_header_name,
            self.config.api_keys,
        );

        let post_json_app = Router::new().route(
            "/echo_a_person",
            post(post_json::echo_a_person::echo_a_person),
        );

        let api_key_protected_app = Router::new()
            .route("/", get(|| async { "API Key Protected" }))
            .route(
                "/do_not_use_extension",
                get(api_key_protected::do_not_use_extension::do_not_use_extension),
            )
            .route(
                "/valid_api_key_from_extension",
                get(api_key_protected::valid_api_key_from_extension::valid_api_key_from_extension),
            )
            .layer(middleware::from_fn_with_state(
                state.clone(),
                validate_api_key_and_put_as_extension::validate_api_key_and_put_as_extension,
            ));

        let app = Router::new()
            .nest("/api_key_protected", api_key_protected_app)
            .nest("/post_json", post_json_app)
            .route("/", get(|| async { "Index" }))
            .route(
                "/extract_api_key_using_extractor",
                get(extract_api_key::extract_api_key_using_extractor),
            )
            .route(
                "/extract_valid_api_key_using_optional_extractor",
                get(extract_valid_api_key_optional::extract_valid_api_key_using_optional_extractor),
            )
            .route(
                "/extract_valid_api_key_using_extractor",
                get(extract_valid_api_key::extract_valid_api_key_using_extractor),
            )
            .layer(middleware::from_fn(trace_headers))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                trace_response_body,
            ))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                method_not_allowed,
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
