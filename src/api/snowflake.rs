use std::sync::{Arc, Mutex};

use chrono::{Utc, TimeZone};


/// Unique id generator.
pub struct Snowflake {
    seq: Mutex<i64>,
}

lazy_static::lazy_static! {
    /// 12 Oct. 2024 as Unix timestamp.
    static ref EPOCH: i64 = Utc.with_ymd_and_hms(2024, 10, 12, 0, 0, 0).unwrap().timestamp_millis();
}


impl Snowflake {
    /// Creates a [`Snowflake`].
    pub fn new() -> Arc<Self> {
        Arc::new(Snowflake { 
            seq: Mutex::new(0) 
        })
    }

    /// Generates unique ids.
    pub fn gen_id(&self) -> i64 {
        static MAX_TIMESTAMP: i64 = 2_i64.pow(42) - 1;
        static MAX_SEQ: i64 = 2_i64.pow(13) - 1;

        let time_since_epoch = (Utc::now().timestamp_millis() - *EPOCH) & MAX_TIMESTAMP;
        let mut seq = self.seq.lock().unwrap();

        let snowflake = (time_since_epoch << 22) | *seq;

        *seq += 1;
        if *seq > MAX_SEQ { *seq = 0 }

        snowflake
    }
}
