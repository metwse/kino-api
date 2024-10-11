use crate::{
    google_signin::GoogleIdToken, 
    server::jwt::{KinoScope, KinoToken}
};
use sqlx::PgPool;
use std::time::{SystemTime, Duration};

pub(super) async fn login_or_signup(token: GoogleIdToken, database: &PgPool) -> KinoToken {
    println!("aa");
    let data = 
        sqlx::query!(
            "SELECT id, username, email FROM users WHERE google_id = $1;",
            token.sub
        )
        .fetch_one(database)
        .await;

    if let Ok(data) = data {
        return KinoToken {
            sub: data.id,
            scope: vec![KinoScope::Auth],
            google_id: token.sub,
            email: data.email,
            username: data.username,
            exp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_secs() + 3600 * 24 * 30,
        }
    }

    todo!()
}


macro_rules! signin {
    ($server:expr) => {
{
    let google_client = Arc::clone(&$server.google_client);
    let pg = Arc::clone(&$server.pg);
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
                    return Json(
                             jwt_client.encode(signin::login_or_signup(token, &pg).await),
                    ).into_response();
                }
            }
        }

        StatusCode::UNAUTHORIZED.into_response()
    }
}
};
}

pub(crate) use signin;
