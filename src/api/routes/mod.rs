/// Sign in route.
mod signin;
/// Users routes.
mod users;

use super::{
    Server,
    jwt::KinoIdToken,
    database::{ORM, BulkRequest, self},
};

use std::{
    time::Duration,
    sync::Arc,
    borrow::Borrow
};

use axum::{
    Router, routing,
    extract::{RawQuery, Path},
    response::IntoResponse,
    Extension,
    http::StatusCode,
    Json
};


impl Server {
    pub(crate) fn routes(self: &'static Arc<Self>) -> Router {
        let orm = ORM::new(Arc::clone(&self.pg), Arc::clone(&self.snowflake));
        
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

        macro_rules! restricted_data {
            ($($ident:ident),*) => { 
                paste::paste! {
                    self.auth(Router::new()
                    $(
                        .merge(
                            self.limit_user(
                                Router::new().route(
                                    &format!("/{}s", stringify!($ident)).to_lowercase()[..],
                                    routing::get({
                                        let orm = orm.clone();
                                        |RawQuery(id): RawQuery| async move {
                                            if let Some(id) = id {
                                                if let Ok(id) = id.parse::<i64>() {
                                                    return Json(orm.[<get_ $ident:lower>](id).await).into_response()
                                                }
                                            }
                                            StatusCode::BAD_REQUEST.into_response()
                                        }
                                    })
                                ),
                                7, Duration::from_secs(2)
                            )
                        )
                        .merge(
                            self.limit_user(
                                Router::new().route(
                                    &format!("/{}s", stringify!($ident)).to_lowercase()[..],
                                    routing::post({
                                        let orm = orm.clone();
                                        |Json(ids): Json<Vec<i64>>| async move {
                                            return Json(orm.[<get_ $ident:lower s>](&ids).await)
                                        }
                                    })
                                ),
                                5, Duration::from_secs(5)
                            )
                        )
                    )*)
                }
            };
        }

        let wn = dict!(self.wordnet);

        let auth_required = routes! {
            get: "/token_info", (5, 5), |Extension(kino_token): Extension<KinoIdToken>| async move { Json(kino_token) };
            get: "/users", (5, 5), users::user!(Arc::clone(&self.pg));
            get: "/wn/get", (5, 2), wn.0;
            get: "/wn/suggest", (3, 5), wn.1;
            get: "/wn/suggest_search", (10, 1), wn.2;
            post: "/bulk", (5, 5), {
                let orm = orm.clone();
                |Json(bulk_request): Json<BulkRequest>| {
                    async move {
                        Json(orm.get(bulk_request).await)
                    }
                }
            };
            get: "/home", (3, 5), {
                let orm = orm.clone();
                |Extension(user): Extension<KinoIdToken>| {
                    async move {
                        Json(orm.home(user.sub).await)
                    }
                }
            };
            get: "/cards/:id/done", (5, 1), {
                let pg = Arc::clone(&self.pg);
                |Path(id): Path<i64>, Extension(user): Extension<KinoIdToken>| {
                    async move {
                        sqlx::query_scalar!(
                            "UPDATE cards SET done_at = NOW() WHERE id = $1 AND owner_id = $2",
                            id, user.sub
                        )
                            .execute(pg.borrow())
                            .await
                            .ok();
                    }
                }
            };
            get: "/cards/:id/move", (5, 1), {
                let pg = Arc::clone(&self.pg);
                |RawQuery(deck_id): RawQuery, Path(id): Path<i64>, Extension(user): Extension<KinoIdToken>| {
                    async move {
                        if let Some(deck_id) = deck_id {
                            if let Ok(deck_id) = deck_id.parse::<i64>() {
                                sqlx::query_scalar!(
                                    r#"UPDATE cards 
                                    SET 
                                        done_at = NOW(), 
                                        deck_id = $3 
                                    WHERE 
                                        id = $1 AND
                                        owner_id = $2 AND 
                                        EXISTS(SELECT 1 FROM decks WHERE id = $3 AND owner_id = $2)"#,
                                    id, user.sub, deck_id
                                )
                                    .execute(pg.borrow())
                                    .await
                                    .ok();
                            }
                        }
                    }
                }
            };
            get: "/cards/:id/delete", (5, 1), {
                let pg = Arc::clone(&self.pg);
                |Path(id): Path<i64>, Extension(user): Extension<KinoIdToken>| {
                    async move {
                        sqlx::query_scalar!(
                            r#"WITH 
                                deleted_faces as (DELETE FROM cards WHERE id = $1 RETURNING front || back AS face_id)
                            DELETE
                                FROM faces
                            WHERE
                                EXISTS(SELECT true FROM deleted_faces WHERE faces.id = ANY(face_id)) AND
                                faces.owner_id = $2"#,
                            id, user.sub
                        )
                            .execute(pg.borrow())
                            .await
                            .ok();
                    }
                }
            };
            post: "/cards/new", (1, 5), {
                let orm = orm.clone();
                |Extension(user): Extension<KinoIdToken>, Json(card_options): Json<database::CreateCard>| {
                    async move {
                        Json(orm.create_card(card_options, user.sub).await)
                    }
                }
            }
        };

        public.merge(auth_required).merge(restricted_data!(Deck, Card, Face, Extension))
    }
}
