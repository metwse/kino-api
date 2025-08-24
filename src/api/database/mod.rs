pub mod structs;

#[allow(unused_imports)]
pub use structs::*;

use std::{borrow::Borrow, sync::Arc};

use serde::{Deserialize, Serialize};

use sqlx::PgPool;

use super::snowflake::Snowflake;

#[derive(Clone)]
pub struct Orm {
    db: Arc<PgPool>,
    snowflake: Arc<Snowflake>,
}

#[derive(Deserialize)]
pub struct CreateCard {
    pub deck_id: i64,
    pub front: (i64, Option<String>),
    pub back: Vec<(i64, Option<String>)>,
}

#[derive(Serialize)]
pub struct CreateCardResponse {
    pub card_id: i64,
    pub front: i64,
    pub back: Vec<i64>,
}

impl Orm {
    /// Creates new Orm.
    pub fn new(db: Arc<PgPool>, snowflake: Arc<Snowflake>) -> Self {
        Self { db, snowflake }
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
        .await
        .is_ok()
    }

    /// Creates card.
    pub async fn create_card(
        &self,
        card_options: CreateCard,
        user_id: i64,
    ) -> Option<CreateCardResponse> {
        sqlx::query_scalar!(
            "SELECT 1 FROM decks WHERE id = $1 AND owner_id = $2",
            card_options.deck_id,
            user_id
        )
        .fetch_one(self.db.borrow())
        .await
        .ok()?;

        if card_options.back.is_empty() {
            return None;
        }

        let front_id = self.snowflake.gen_id();

        let mut query = format!(
            r#"
            INSERT INTO faces
            VALUES
                ({}, $1, $2, $3),
        "#,
            front_id
        );

        let mut back_ids = Vec::with_capacity(card_options.back.len());

        for a in 0..card_options.back.len() {
            let back_id = self.snowflake.gen_id();
            back_ids.push(back_id);
            query += &format!("({}, $1, ${}, ${})", back_id, a * 2 + 4, a * 2 + 5)[..];
            if a != card_options.back.len() - 1 {
                query += ", "
            };
        }

        let mut query = sqlx::query(&query[..])
            .bind(user_id)
            .bind(card_options.front.0)
            .bind(&card_options.front.1);

        for back_face in card_options.back {
            query = query.bind(back_face.0).bind(back_face.1);
        }

        query.fetch_optional(self.db.borrow()).await.ok()?;

        let card_id = self.snowflake.gen_id();
        sqlx::query!(
            r#"
                INSERT INTO cards
                SELECT $1, $2, $3, $4, $5, NULL;
            "#,
            card_id,
            user_id,
            card_options.deck_id,
            front_id,
            &*back_ids
        )
        .fetch_optional(self.db.borrow())
        .await
        .ok()?;

        Some(CreateCardResponse {
            card_id,
            front: front_id,
            back: back_ids,
        })
    }

    pub async fn home(&self, user_id: i64) -> Option<HomeResponse> {
        let decks = sqlx::query_scalar!(
            "SELECT array_agg(id) as \"arr!\" FROM decks WHERE owner_id = $1",
            user_id
        )
        .fetch_one(self.db.borrow())
        .await
        .ok()?;

        let cards = sqlx::query!(
            "SELECT id, deck_id, done_at FROM cards WHERE owner_id = $1",
            user_id
        )
        .fetch_all(self.db.borrow())
        .await
        .ok()?;

        let cards = cards
            .into_iter()
            .map(|row| (row.id, row.deck_id, row.done_at))
            .collect::<Vec<_>>();

        Some(HomeResponse { decks, cards })
    }
}

#[derive(Serialize)]
pub struct HomeResponse {
    decks: Vec<i64>,
    cards: Vec<(i64, i64, Option<chrono::NaiveDateTime>)>,
}

macro_rules! struct_defs {
    ($($struct:ident, $limit:expr);*) => {
        paste::paste! {
            impl Orm {
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
                    pub async fn [<get_ $struct:lower s>](&self, id: &[i64]) -> Vec<$struct> {
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
