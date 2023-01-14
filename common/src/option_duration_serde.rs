use chrono::Duration;
use serde::{Deserialize, Deserializer, Serializer};

pub fn serialize<S>(duration: &Option<Duration>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(ref d) = *duration {
        return s.serialize_i64(d.num_milliseconds());
    }
    s.serialize_none()
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    let milliseconds: Option<i64> = Option::deserialize(deserializer)?;
    if let Some(milliseconds) = milliseconds {
        return Ok(Some(Duration::milliseconds(milliseconds)));
    }

    Ok(None)
}
