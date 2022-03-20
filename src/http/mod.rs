use std::{net::SocketAddr, str::FromStr};

use axum::{
    body,
    extract::Extension,
    http::{header, HeaderValue},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use tower::ServiceBuilder;
use tower_http::{trace::TraceLayer, ServiceBuilderExt};

use crate::{
    backends::github::middleware::VerifyGitHubSignatureLayer,
    config::Config,
    error::{ErrorCode, ErrorCodeDetail},
    server_info::ServerInfo,
    service::ServiceHandler,
};

impl IntoResponse for ErrorCode {
    fn into_response(self) -> Response {
        let details: ErrorCodeDetail = (&self).into();
        let json_data = serde_json::to_string(&details).unwrap();
        let body = body::boxed(body::Full::from(json_data));

        Response::builder()
            .status(details.status_code())
            .header("Content-Type", "application/json")
            .body(body)
            .unwrap()
    }
}

#[tracing::instrument]
async fn root() -> Json<ServerInfo> {
    Json(ServerInfo::new())
}

#[tracing::instrument]
pub async fn start_server(config: Config, services: ServiceHandler) -> color_eyre::Result<()> {
    let addr = SocketAddr::from_str(config.bind_ip()).unwrap();
    let app = build_http_router(config, services);
    tracing::info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

pub(crate) fn build_http_router(config: Config, services: ServiceHandler) -> Router {
    let middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .insert_response_header_if_not_present(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

    Router::new()
        .route("/", get(root))
        .route(
            "/webhook/github",
            post(super::backends::github::webhook).layer(VerifyGitHubSignatureLayer::new(
                config.webhook_secret().map(|x| x.to_owned()),
            )),
        )
        .layer(middleware.into_inner())
        .layer(Extension(config))
        .layer(Extension(services))
}
