use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Extension, Router
};
use redis::{
    Commands,
    Client as RedisClient
};

use std::{
    sync::{Arc, Mutex},
    time::Duration
};

use crate::server::Server;

use tower::ServiceBuilder;

macro_rules! limit_middleware_fn {
    (@define $fn:ident, $header:expr) => {
        paste::paste! {
            async fn [<limit_middleware_ $fn>](Extension(limiter_options): Extension<LimitOptions>, request: Request, next: Next) -> Response {
                let Some(ip) = (if let Some(header) = request.headers().get($header) {
                        header.to_str().ok()
                    } else { None })
                    else {
                        return StatusCode::BAD_REQUEST.into_response()
                    };

                let key = format!("ip:{ip}");

                let redis = limiter_options.database;
                let rate_limit: i32 = redis::transaction(&mut redis.lock().unwrap(), &[&key[..]], |con, pipe| {
                    let key = &key[..];
                    let limit: Option<usize> = con.get(key)?;
                    // increment limit by one or signal limit exceeded if exists
                    if let Some(limit) = limit {
                        if limit >= limiter_options.num {
                            if let Some(ttl) = pipe.ttl(key).query::<Option<Vec<i32>>>(con)? {
                                if ttl.len() > 0 {
                                    return Ok(Some(ttl[0]))
                                }
                            }
                        } else {
                            pipe.incr(key, 1).ignore().query::<()>(con)?;
                        }
                    } else {
                        pipe
                            .set(key, 1).ignore()
                            .expire(key, limiter_options.per.as_secs().try_into().unwrap()).ignore()
                            .query::<()>(con)?;
                    }
                    Ok(Some(-2))
                }).ok().unwrap_or(-2);


                if rate_limit > 0 {
                    let mut response = (StatusCode::TOO_MANY_REQUESTS, rate_limit.to_string()).into_response();
                    response.headers_mut().insert("Retry-After", HeaderValue::from_str(&rate_limit.to_string()[..]).unwrap());
                    response.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));
                    response
                } else {
                    let response = next.run(request).await;
                    response
                }
            }
        }
        limit_middleware_fn!(@method $fn);
    };
    (@method $fn:ident) => {
        paste::paste! {
            impl Server {
                pub(crate) fn [<limit_ $fn>](&mut self, router: Router, num: usize, per: Duration) -> Router {
                    let limiter_options = LimitOptions {
                        num, per,
                        database: Arc::clone(&self.redis)
                    };

                    router.route_layer(
                        ServiceBuilder::new()
                        .layer(Extension(limiter_options))
                        .layer(middleware::from_fn(limit_middleware_ip))
                    )
                }
            }
        }
    }
}

#[derive(Clone)]
pub(super) struct LimitOptions {
    num: usize,
    per: Duration,
    database: Arc<Mutex<RedisClient>>
}

limit_middleware_fn!(@define ip, "X-Real-IP");
limit_middleware_fn!(@define token, "Token");