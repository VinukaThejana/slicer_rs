use ::log::info;
use axum::{
    Router,
    http::{Method, header},
    middleware,
    routing::{get, post},
};
use slicer_rs::{
    api_docs::ApiDoc,
    config::{ENV, log},
    handler,
    util::{governor_conf, governor_err, shutdown},
};
use std::{net::SocketAddr, time::Duration};
use tokio::{net::TcpListener, time};
use tower::ServiceBuilder;
use tower_governor::GovernorLayer;
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log::setup();

    let governor_conf = governor_conf();
    let governor_limiter = governor_conf.limiter().clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            info!(
                "[Governor] limiting storage size : {}",
                governor_limiter.len()
            );
            governor_limiter.retain_recent();
        }
    });
    let governor_layer = GovernorLayer::new(governor_conf).error_handler(governor_err);

    let app = Router::new()
        .route("/health", get(handler::health))
        .route(
            "/volume",
            post(handler::model::calculate_volume).route_layer(middleware::from_fn(
                slicer_rs::middleware::auth::access_token,
            )),
        )
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(Duration::from_secs(10 * 60)))
                .layer(
                    CorsLayer::new()
                        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
                        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                        .allow_origin(Any),
                ),
        )
        .layer(governor_layer);

    let router = Router::new()
        .merge(SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api", app);

    info!("up and running on : {}", &ENV.port);
    axum::serve(
        TcpListener::bind(&format!("0.0.0.0:{}", &ENV.port))
            .await
            .unwrap(),
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown())
    .await
    .unwrap();

    anyhow::Ok(())
}
