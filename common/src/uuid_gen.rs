use std::convert::TryInto;

use chrono::Utc;
use uuid::{Bytes, Uuid};

pub fn uuid_time_nextval() -> Uuid {
    let prefix: u16 = ((Utc::now().timestamp_millis() / 60) % 65536)
        .try_into()
        .unwrap();
    let random_uuid_bytes = Uuid::new_v4();
    let random_uuid_bytes = random_uuid_bytes.as_bytes();

    let mut uuid_bytes: Bytes = [0; 16];
    let prefix_bytes = prefix.to_be_bytes();
    for i in 0..16 {
        if i < 2 {
            uuid_bytes[i] = prefix_bytes[i];
        } else {
            uuid_bytes[i] = random_uuid_bytes[i];
        }
    }

    Uuid::from_bytes(uuid_bytes)
}
