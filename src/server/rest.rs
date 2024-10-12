use axum::{
    error_handling::HandleErrorLayer, 
    http::StatusCode,
    response::IntoResponse,
    routing,
    Router,
};

use tower::{ServiceBuilder, BoxError};
use tower_http::trace::TraceLayer;

use std::{
    time::{Duration, Instant},
    sync::Arc
};

use super::Server;

impl Server {
    pub async fn serve(self: &'static Arc<Self>, host: &str) {
        let uptime = Instant::now();

        let app = Router::new()
            .route("/", routing::get(move ||
                async move {
                    axum::Json(uptime.elapsed().as_secs())
                }
            ))
            .merge(self.routes())
            .layer(ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|_: BoxError| async move {
                    StatusCode::REQUEST_TIMEOUT.into_response()
                }))
                .timeout(Duration::from_secs(16))
                .into_inner()
            )
            .layer(TraceLayer::new_for_http());

        let listener = tokio::net::TcpListener::bind(host)
            .await.expect(&format!("Cannot bind {host}"));

        axum::serve(listener, app).await.unwrap()
    }
}
