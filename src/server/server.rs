use std::{
    path::PathBuf,
    sync::{Arc, Mutex}
};

use sqlx::{Pool, postgres::{
    PgPoolOptions,
    Postgres
}};

use crate::dicts::WordNetDatabase;

use crate::google_signin::GoogleClient;

use redis::Client;

/// Kino api web server struct for shared objects.
pub struct Server {
    pub(crate) wordnet: Arc<WordNetDatabase>,
    pub(crate) google_client: Arc<GoogleClient>,
    pub(crate) pg: Arc<Pool<Postgres>>,
    pub(crate) redis: Arc<Mutex<Client>>,
}

pub struct ServerBuilder<'a> {
    pub google_audiences: Vec<String>,
    pub google_allowed_hosted_domains: Vec<String>,
    pub wn_location: &'a str,
    pub pg_url: &'a str,
    pub redis_url: &'a str,
}

impl<'a> ServerBuilder<'a> {
    /// Builds [`Server`] from [`ServerBuilder`]
    pub async fn build(self) -> Server {
        let wordnet = Arc::new(WordNetDatabase::new(PathBuf::from(self.wn_location)));

        let pg = Arc::new(PgPoolOptions::new()
            .max_connections(8)
            .connect(self.pg_url)
            .await.expect("Cannot connect to Postgres database."));

        let redis = redis::Client::open(self.redis_url).expect("Cannot connect to Postgres database.");

        let mut google_client = GoogleClient::new(self.google_audiences, self.google_allowed_hosted_domains);
        google_client.init().await;

        Server {
            wordnet, 
            google_client: Arc::new(google_client),
            pg,
            redis: Arc::new(Mutex::new(redis))
        }
    }
}

