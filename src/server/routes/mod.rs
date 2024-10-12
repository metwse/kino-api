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
    pub(crate) fn routes(self: &'static Arc<Self>) -> Router {
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

        macro_rules! dict {
            ($database:expr) => {
                {
                    use std::sync::Arc;
                    use crate::dicts::Database;

                    use axum::{
                        extract::RawQuery,
                        http::StatusCode,
                        Json,
                        response::IntoResponse
                    };
                    (
                        dict!($database, get, 24),
                        dict!($database, suggest, 20),
                        dict!($database, suggest_search, 24),
                    )
                }
            };
            ($database:expr, $fn:ident, $len:expr) => {
                |RawQuery(query): RawQuery| {
                    let database = Arc::clone(&$database);
                    async move {
                        if let Some(ref query) = query {
                            if query.len() < $len {
                                return Json(database.$fn(&query)).into_response()
                            }
                        }

                        StatusCode::BAD_REQUEST.into_response()
                    }
                }
            }
        }

        let wn = dict!(self.wordnet);

        let auth_required = routes! {
            get: "/token_info", (5, 5), |Extension(kino_token): Extension<KinoToken>| async move { Json(kino_token) };
            get: "/wn/get", (5, 2), wn.0;
            get: "/wn/suggest", (3, 5), wn.1;
            get: "/wn/suggest_search", (5, 1), wn.2;
        };

        public.merge(auth_required)
    }
}
