use std::{
    path::PathBuf,
    sync::Arc
};

use sqlx::{Pool, postgres::{
    PgPoolOptions,
    Postgres
}};

use crate::dicts::WordNetDatabase;

use crate::google_signin::GoogleClient;


/// Kino api web server struct for shared objects.
pub struct Server {
    wordnet: Arc<WordNetDatabase>,
    google_client: Arc<GoogleClient>,
    pg: Arc<Pool<Postgres>>,
}

pub struct ServerBuilder<'a> {
    pub google_audiences: Vec<String>,
    pub google_allowed_hosted_domains: Vec<String>,
    pub wn_location: &'a str,
    pub pg_url: &'a str,
}

impl<'a> ServerBuilder<'a> {
    /// Builds [`Server`] from [`ServerBuilder`]
    pub async fn build(self) -> Server {
        let wordnet = Arc::new(WordNetDatabase::new(PathBuf::from(self.wn_location)));
        let pg = Arc::new(PgPoolOptions::new()
            .max_connections(8)
            .connect(self.pg_url)
            .await.expect("Cannot connect postgres database."));
        let mut google_client = GoogleClient::new(self.google_audiences, self.google_allowed_hosted_domains);
        google_client.init().await;

        Server {
            wordnet, 
            google_client: Arc::new(google_client),
            pg
        }
    }
}

