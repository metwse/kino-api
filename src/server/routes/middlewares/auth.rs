use crate::server::Server;

use std::sync::Arc;

use axum::{
    Router,
    extract::Request,
    response::{Response, IntoResponse},
    http::{StatusCode, HeaderValue},
    middleware::{self, Next},
    Extension
};
use crate::server::jwt::KinoScope;

async fn auth(Extension(server): Extension<Arc<Server>>, mut request: Request, next: Next) -> Response {
    let Some(token) = (if let Some(header) = request.headers().get("Token") {
        header.to_str().ok()
    } else { None }) else { return StatusCode::BAD_REQUEST.into_response() };

    if let Some(kino_token) = server.jwt_client.decode(token) {
        if kino_token.scope.contains(&KinoScope::Auth) {
            request.headers_mut().insert("UserId", HeaderValue::try_from(kino_token.sub).unwrap());
            request.extensions_mut().insert(kino_token);
            return next.run(request).await
        }
    } 

    StatusCode::UNAUTHORIZED.into_response()
}

impl Server {
    pub(crate) fn auth(self: &Arc<Self>, router: Router) -> Router {
        router
            .layer(middleware::from_fn(auth))
            .layer(Extension(Arc::clone(&self)))
    }
}
