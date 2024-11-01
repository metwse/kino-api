use crate::api::{
    Server,
    jwt::KinoTokenScope
};

use std::sync::Arc;

use axum::{
    Router,
    extract::Request,
    response::{Response, IntoResponse},
    http::{StatusCode, HeaderValue},
    middleware::{self, Next},
    Extension
};


async fn auth(Extension(server): Extension<Arc<Server>>, mut request: Request, next: Next) -> Response {
    let Some(token) = (
        if let Some(header) = request.headers().get("Token") {
            header.to_str().ok()
        } else { None }
    ) else { 
        return StatusCode::UNAUTHORIZED.into_response() 
    };

    if let Some(kino_token) = server.kino_client.decode(token) {
        if kino_token.scope.contains(&KinoTokenScope::Auth) {
            tracing::debug!("User authenticated: id={} email={}", kino_token.sub, kino_token.email);

            request
                .headers_mut()
                .insert("UserId", HeaderValue::from(kino_token.sub));

            request
                .extensions_mut()
                .insert(kino_token);

            return next.run(request).await
        }
    } 

    StatusCode::UNAUTHORIZED.into_response()
}

impl Server {
    /// Authentication with `Token` header.
    pub(crate) fn auth(self: &Arc<Self>, router: Router) -> Router {
        router
            .layer(middleware::from_fn(auth))
            .layer(Extension(Arc::clone(self)))
    }
}
