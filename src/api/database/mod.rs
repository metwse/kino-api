pub mod structs;

#[allow(unused_imports)]
pub use structs::*;

use std::{ 
    borrow::Borrow, 
    sync::Arc,
};

use serde::{Serialize, Deserialize};

use sqlx::PgPool;

use super::snowflake::Snowflake;

#[derive(Clone)]
pub struct ORM {
    db: Arc<PgPool>,
    snowflake: Arc<Snowflake>
}

impl ORM {
    /// Creates new ORM.
    pub fn new(db: Arc<PgPool>, snowflake: Arc<Snowflake>) -> Self {
        Self { 
            db,
            snowflake
        }
    }

    /// Assigns default decks to user.
    pub async fn default_decks(&self, user_id: i64) -> bool {
        sqlx::query!(
            r#"
                INSERT INTO decks 
                VALUES
                    ($2, $1, 0, INTERVAL '12 hours', 0),
                    ($3, $1, 0, INTERVAL '1 day', 1),
                    ($4, $1, 0, INTERVAL '2 days', 2),
                    ($5, $1, 0, INTERVAL '4 days', 3),
                    ($6, $1, 0, INTERVAL '9 days', 4),
                    ($7, $1, 0, INTERVAL '14 days', 5)
            "#,
            user_id,
            self.snowflake.gen_id(),
            self.snowflake.gen_id(),
            self.snowflake.gen_id(),
            self.snowflake.gen_id(),
            self.snowflake.gen_id(),
            self.snowflake.gen_id(),
        )
            .fetch_one(self.db.borrow())
            .await.is_ok()
    }

    pub async fn home(&self, user_id: i64) -> Option<HomeResponse> {
        let decks = sqlx::query_scalar!(
            "SELECT array_agg(id) as \"arr!\" FROM decks WHERE owner_id = $1",
            user_id
        )
            .fetch_one(self.db.borrow())
            .await.ok()?;

        let cards = sqlx::query!(
            "SELECT id, done_at FROM cards WHERE owner_id = $1",
            user_id
        )
            .fetch_all(self.db.borrow())
            .await.ok()?;

        let cards = cards.into_iter().map(|row| (row.id, row.done_at)).collect::<Vec<_>>();

        Some(HomeResponse {
            decks,
            cards
        })
    }
}

#[derive(Serialize)]
pub struct HomeResponse {
    decks: Vec<i64>,
    cards: Vec<(i64, Option<chrono::NaiveDateTime>)>,
}

macro_rules! struct_defs {
    ($($struct:ident, $limit:expr);*) => {
        paste::paste! {
            impl ORM {
                //const STRUCT_COUNT: u8 = struct_defs!(@count $($struct,)*);

                $(
                    #[doc="Gets [`" $struct "`]."]
                    pub async fn [<get_ $struct:lower>](&self, id: i64) -> Option<$struct> {
                        let query = concat!("SELECT * FROM ", stringify!($struct), "s WHERE id = ");
                        let data: Option<$struct> = 
                            sqlx::query_as(&format!("{query}{id}")[..])
                                .fetch_one(self.db.borrow())
                                .await.ok();
                        data
                    }

                    #[doc="Gets vector of [`" $struct "`]."]
                    pub async fn [<get_ $struct:lower s>](&self, id: &Vec<i64>) -> Vec<$struct> {
                        let query = concat!("SELECT * FROM ", stringify!($struct), "s WHERE id = ");
                        let data: Vec<$struct> =
                            sqlx::query_as(
                                    &format!(
                                        "{query}ANY(ARRAY[{}])", 
                                        id
                                            .iter().take($limit)
                                            .map(|x| x.to_string())
                                            .collect::<Vec<String>>()
                                            .join(",")
                                    )[..]
                                )
                                .fetch_all(self.db.borrow())
                                .await.unwrap_or(vec![]);
                        data
                    }
                )*

                #[doc="Gets whatever object in a single query."]
                pub async fn get(&self, request: BulkRequest) -> BulkResponse {
                    $(
                        let [<$struct:lower s>] = 
                            if let Some(ids) = &request.[<$struct:lower s>] {
                                self.[<get_ $struct:lower s>](ids).await
                            } else { vec![] };
                    )*

                    BulkResponse {
                        $([<$struct:lower s>],)*
                    }
                }
            }

            #[derive(Debug, Serialize, Deserialize)]
            pub struct BulkResponse {
                $(
                    pub [<$struct:lower s>]: Vec<$struct>,
                )*
            }

            #[derive(Serialize, Deserialize)]
            pub struct BulkRequest {
                $(
                    pub [<$struct:lower s>]: Option<Vec<i64>>,
                )*
            }
        }
    };
    (@count $($ident:ident),*,) => { 0 $(+ struct_defs!(@innercount $ident))* };
    (@innercount $ident:ident) => { 1 }
}

struct_defs!(
    Deck, 16;
    Card, 64;
    Face, 192;
    Extension, 8
);
