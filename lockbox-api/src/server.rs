use std::{sync::Arc, time::Duration, net::SocketAddr};
use axum::{
    Router,
    Extension,
    body::Body,
};
use http::{Request, Response};
use tower_http::{
    trace::{TraceLayer, DefaultMakeSpan},
};
use tower::ServiceBuilder;
use axum::response::{Json, IntoResponse};
use axum_server::{Handle, tls_rustls::RustlsConfig};
use utoipa::{ToSchema, OpenApi};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use serde::Serialize;
use tokio::signal;

use lockbox_core::telemetry::{info, Span};

use crate::{
    dto::v1::common::ErrorResponseDTO,
    router::v1,
    openapi::ApiDoc,
    error::ApiError,
    state::ApiState,
    constants::APP_NAME,
};


#[derive(Serialize, ToSchema)]
struct PingResponse {
    name: &'static str,
    version: &'static str,
}

#[utoipa::path(
    get,
    path = "",
    operation_id = "ping",
    tag = "ping",
    responses(
        (status = 200, description = "Ping endpoint", body = PingResponse),
    ),
)]
async fn ping_endpoint() -> impl IntoResponse {
    Json(PingResponse {
        name: APP_NAME,
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn not_found_fallback() -> impl IntoResponse {
    ErrorResponseDTO::from(ApiError::not_found("not found"))
        .into_response()
}

pub fn create_router(api_state: Arc<ApiState>) -> Router {
    let (router, api_doc) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/v1", v1::router())
        .layer(Extension(api_state))
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(
                            DefaultMakeSpan::new()
                                .include_headers(false)
                        )
                        .on_request(
                            |request: &Request<Body>, span: &Span| {
                                span
                                    .record(
                                        "method", 
                                        &request
                                            .method()
                                            .as_str()
                                    )
                                    .record(
                                        "uri", 
                                        &request
                                            .uri()
                                            .to_string()
                                    );

                                info!(
                                    parent: span,
                                    event = "ReceivedRequest",
                                    method = %request.method(),
                                    uri = %request.uri(),
                                );
                            }
                        )
                        .on_response(
                            |response: &Response<Body>, latency: Duration, span: &Span| {
                                info!(
                                    parent: span,
                                    event = "RequestFinished",
                                    status = %response.status(),
                                    latency = ?latency,
                                );
                            }
                        )
                )
        )
        // Ping endpoint after the tracing layer to ensure
        // that it is not traced
        .routes(routes!(ping_endpoint))
        .fallback(not_found_fallback)
        .split_for_parts();

    let router = router
        .merge(Scalar::with_url("/.well-known/docs", api_doc));

    router
}

pub async fn create_tls_config(cert_file: String, key_file: String) -> RustlsConfig {
    RustlsConfig::from_pem_file(cert_file, key_file)
        .await
        .expect("Failed to create TLS config")
}

pub async fn serve_tls(addr: String, router: Router, tls_config: RustlsConfig) -> std::io::Result<()> {
    let handle = Handle::new();
    tokio::spawn(shutdown_signal(handle.clone()));
    axum_server::bind_rustls(addr.parse::<SocketAddr>().expect("Invalid address"), tls_config)
        .handle(handle)
        .serve(router.into_make_service())
        .await
}

pub async fn serve(addr: String, router: Router) -> std::io::Result<()> {
    let handle = Handle::new();
    tokio::spawn(shutdown_signal(handle.clone()));
    axum_server::bind(addr.parse::<SocketAddr>().expect("Invalid address"))
        .handle(handle)
        .serve(router.into_make_service())
        .await
}

pub async fn shutdown_signal(handle: axum_server::Handle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => (),
        _ = terminate => (),
    }

    handle.graceful_shutdown(Some(Duration::from_secs(10)));
}