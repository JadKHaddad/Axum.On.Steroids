use std::{net::SocketAddr, path::Path};

use anyhow::Context;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
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
        method_not_allowed::method_not_allowed, not_found, trace_headers::trace_headers,
        trace_response_body::trace_response_body, validate_api_key_and_put_as_extension,
    },
    openid_configuration::OpenIdConfiguration,
    route::{
        api_key_protected, books, extract_api_key, extract_authenticated_basic_auth,
        extract_basic_auth, extract_bearer_token, extract_jwt_claims, extract_valid_api_key,
        extract_valid_api_key_optional, post_json, validated,
    },
    state::ApiState,
    types::{used_api_key::UsedApiKey, used_basic_auth::UsedBasicAuth},
};

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    socket_address: SocketAddr,
    error_verbosity: ErrorVerbosity,
    api_key_header_name: String,
    api_keys: Vec<UsedApiKey>,
    basic_auth_users: Vec<UsedBasicAuth>,
    openid_configuration_url: String,
    jwks_time_to_live_in_seconds: u64,
    audience: Vec<String>,
}

impl ServerConfig {
    pub async fn from_config_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let config_file = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read config file")?;

        let config: ServerConfig =
            serde_yaml::from_str(&config_file).context("Failed to parse config file")?;

        Ok(config)
    }
}

pub struct Server {
    config: ServerConfig,
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    async fn obtain_openid_config(
        &self,
        http_client: &reqwest::Client,
    ) -> anyhow::Result<OpenIdConfiguration> {
        let openid_config = http_client
            .get(&self.config.openid_configuration_url)
            .send()
            .await
            .context("Failed to get OpenID configuration")?
            .json::<OpenIdConfiguration>()
            .await
            .context("Failed to parse OpenID configuration")?;

        Ok(openid_config)
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let http_client = reqwest::Client::new();

        tracing::trace!("Obtaining OpenID configuration");
        let openid_config = self.obtain_openid_config(&http_client).await?;
        tracing::debug!(?openid_config, "Obtained OpenID configuration");

        let state = ApiState::new(
            http_client,
            self.config.error_verbosity,
            self.config.api_key_header_name,
            self.config.api_keys,
            self.config.basic_auth_users,
            openid_config,
            self.config.jwks_time_to_live_in_seconds,
            self.config.audience,
        )
        .await
        .context("Failed to create state")?;

        let books_app = Router::new()
            .route("/get_book", get(books::get_book::get_book))
            .route(
                "/get_book_not_found",
                get(books::get_book::get_book_not_found),
            )
            .route(
                "/get_book_id_too_big",
                get(books::get_book::get_book_id_too_big),
            );

        let post_json_app = Router::new().route(
            "/echo_a_person",
            post(post_json::echo_a_person::echo_a_person),
        );

        let validate_app = Router::new().route(
            "/validate_a_person",
            post(validated::validate_a_person::validate_a_person),
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
            .fallback(not_found::not_found)
            .nest("/api_key_protected", api_key_protected_app)
            .nest("/post_json", post_json_app)
            .nest("/validated", validate_app)
            .nest("/books", books_app)
            .route("/", get(|| async { "Index" }))
            .route(
                "/extract_valid_jwt_claims_using_extractor",
                get(extract_jwt_claims::extract_valid_jwt_claims_using_extractor),
            )
            .route(
                "/extract_bearer_token_using_extractor",
                get(extract_bearer_token::extract_bearer_token_using_extractor),
            )
            .route(
                "/extract_authenticated_basic_auth_using_extractor",
                get(extract_authenticated_basic_auth::extract_authenticated_basic_auth_using_extractor),
            )
            .route(
                "/extract_basic_auth_using_extractor",
                get(extract_basic_auth::extract_basic_auth_using_extractor),
            )
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
