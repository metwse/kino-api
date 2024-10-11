mod middlewares;
mod signin;

use std::{
    time::Duration,
    sync::Arc
};

use axum::{
    Router, routing,
    Extension,
    Json
};

use super::Server;

use super::jwt::KinoToken;


impl Server {
    pub(crate) fn routes(self: &Arc<Self>) -> Router {
        let public = self.limit_ip(
            Router::new().route("/signin", routing::get(signin::signin!(Arc::clone(&self)))),
            5, Duration::from_secs(5)
        );

        macro_rules! routes {
            {
                $($type:ident: $route:expr, ($num:expr, $per:expr), $fn:expr);* $(;)?
            } => {
               self.auth(Router::new()
                $(
                    .merge(
                        self.limit_user(
                            Router::new().route($route, routing::$type($fn)),
                            $num, Duration::from_secs($per)
                        )
                    )
                )*)
            };
        }

        let auth_required = routes! {
            get: "/test", (5, 5), |Extension(kino_token): Extension<KinoToken>| async move { Json(kino_token) };
        };

        public.merge(auth_required)
    }
}
