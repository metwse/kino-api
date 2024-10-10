mod middlewares;

use std::time::Duration;

use axum::{
    Router, routing,
};

use super::Server;

impl Server {
    pub(crate) fn routes(&mut self) -> Router {
        self.limit_ip(
            Router::new().route("/signin", routing::get(|| async move { "TODO" })),
            5, Duration::from_secs(5)
        )
    }
}
