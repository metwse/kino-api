use crate::{
    google_signin::GoogleIdToken, 
    server::{
        jwt::{KinoScope, KinoToken},
        snowflake::Snowflake
    }
};
use sqlx::PgPool;
use std::{
    time::{SystemTime, Duration},
    sync::Arc
};

pub(super) async fn login_or_signup(token: GoogleIdToken, database: &PgPool, snowflake: &Arc<Snowflake>) -> Option<KinoToken> {
    let exp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_secs() + 3600 * 24 * 30;
    let data = 
        sqlx::query!(
            "SELECT id, username, email FROM users WHERE google_id = $1;",
            token.sub
        )
        .fetch_one(database)
        .await;

    if let Ok(data) = data {
        return Some(KinoToken {
            sub: data.id,
            scope: vec![KinoScope::Auth],
            google_id: token.sub,
            email: data.email,
            username: data.username,
            exp 
        })
    }

    let id = snowflake.gen_id();
    let Ok(_) = 
        sqlx::query_scalar!(
            r#" 
            INSERT INTO users SELECT $1, $2, $3, NULL, $4, $5 WHERE 
                NOT EXISTS (SELECT 1 FROM users WHERE id = $1) AND
                NOT EXISTS (SELECT 1 FROM users WHERE email = CAST($2 AS character varying(254))) AND
                NOT EXISTS (SELECT 1 FROM users WHERE google_id = CAST($3 AS text))
                RETURNING true
            "#,
            id, token.email, token.sub, token.name, token.picture
        ).fetch_one(database)
        .await else { return None };

    Some(KinoToken {
        sub: id,
        scope: vec![KinoScope::Auth],
        google_id: token.sub,
        email: token.email.unwrap(),
        username: None,
        exp
    })
}


macro_rules! signin {
    ($server:expr) => {
{
    let google_client = Arc::clone(&$server.google_client);
    let pg = Arc::clone(&$server.pg);
    let snowflake = Arc::clone(&$server.snowflake);

    let jwt_client = Arc::clone(&$server.jwt_client);
    use std::collections::HashMap;

    use axum::{
        extract::Query,
        response::IntoResponse,
        http::StatusCode,
        Json
    };

    |Query(query): Query<HashMap<String, String>>| async move { 
        let Some(token) = &query.get("token") else {
            return StatusCode::BAD_REQUEST.into_response()
        };

        if let Ok(token) = google_client.validate(&token) {
            if let Some(email_verified) = token.email_verified {
                if email_verified && token.email.is_some() {
                    if let Some(kino_token) = signin::login_or_signup(token, &pg, &snowflake).await {
                        return Json(
                            jwt_client.encode(kino_token),
                        ).into_response();
                    }
                }
            }
        }

        StatusCode::UNAUTHORIZED.into_response()
    }
}
};
}

pub(crate) use signin;
