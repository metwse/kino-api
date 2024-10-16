use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: Option<String>,
    pub username: Option<String>,
    pub picture: Option<String>,
}


macro_rules! user {
    ($database:expr) => {
        {
            use axum::{
                extract::Query,
                http::StatusCode,
                response::IntoResponse,
                Json
            };

            use serde::Deserialize;

            use std::borrow::Borrow;

            #[derive(Deserialize)]
            struct UserQuery {
                username: Option<String>,
                id: Option<i64>,
            }

            type User = users::User;

            move |Query(query): Query<UserQuery>| 
                async move {
                    let data = if let Some(id) = query.id {
                        sqlx::query_as!(
                                User,
                                "SELECT id, username, name, picture FROM users WHERE id = $1",
                                id
                            )
                            .fetch_one($database.borrow())
                            .await
                    } else if let Some(username) = query.username {
                        sqlx::query_as!(
                                User,
                                "SELECT id, username, name, picture FROM users WHERE username = $1",
                                username
                            )
                            .fetch_one($database.borrow())
                            .await
                    } else {
                        return StatusCode::BAD_REQUEST.into_response()
                    };

                    if let Ok(data) = data {
                        Json(data).into_response()
                    } else {
                        StatusCode::BAD_REQUEST.into_response()
                    }
                }
        }
    };
}

pub(crate) use user;
