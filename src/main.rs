use ::log::info;
use axum::{
    Router,
    http::{Method, header},
    routing::get,
};
use slicer_rs::{
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
        .nest(
            "/api",
            Router::new()
                .route("/health", get(handler::health))
                .route("/volume", get(handler::model::calculate_volume)),
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

    info!("up and running on : {}", &ENV.port);
    axum::serve(
        TcpListener::bind(&format!("0.0.0.0:{}", &ENV.port))
            .await
            .unwrap(),
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown())
    .await
    .unwrap();

    anyhow::Ok(())
}
