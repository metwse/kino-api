use kino_api::api::ServerBuilder;

use lazy_static::lazy_static;

use dotenv::dotenv;

// loads environment variables to &'static str
macro_rules! env {
    ($($name: ident),*) => {
        lazy_static! {
            $(
                static ref $name: &'static str = Box::leak(
                    std::env::var(stringify!($name))
                        .expect(&format!("Cannot find environment variable {}", stringify!($name))[..])
                        .into_boxed_str()
                );
            )*
        }
    };
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env![
        HOST,
        GOOGLE_CLIENT_ID,
        WN_DATABASE,
        DATABASE_URL,
        REDIS_URL,
        JWT_SECRET
    ];

    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let server = ServerBuilder {
        google_audiences: vec![String::from(*GOOGLE_CLIENT_ID)],
        google_allowed_hosted_domains: vec![],
        wn_location: *WN_DATABASE,
        pg_url: *DATABASE_URL,
        redis_url: *REDIS_URL,
        jwt_secret: *JWT_SECRET,
    }
    .build()
    .await;

    let server = Box::leak(Box::new(server));

    server.serve(*HOST).await
}
