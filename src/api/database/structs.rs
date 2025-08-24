use serde::{Deserialize, Serialize};
use sqlx::{postgres::types::PgInterval, FromRow};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Deck {
    pub id: i64,
    pub owner_id: i64,
    pub card_count: i64,
    #[serde(with = "PgIntervalRemote")]
    pub interval: PgInterval,
    pub level: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(remote = "PgInterval")]
struct PgIntervalRemote {
    pub months: i32,
    pub days: i32,
    pub microseconds: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Card {
    pub id: i64,
    pub owner_id: i64,
    pub deck_id: i64,
    pub front: i64,
    pub back: Vec<i64>,
    pub done_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Face {
    pub id: i64,
    pub owner_id: i64,
    pub extension_id: Option<i64>,
    pub data: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Extension {
    pub id: i64,
    pub owner_id: Option<i64>,
    pub name: String,
    pub data: String,
}
