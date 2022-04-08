use std::fmt;
use serde::{de::{Deserializer, Visitor}, Serializer };

use chrono::Duration;
use serde::de::{Error};


pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
    where D: Deserializer<'de>
{
    d.deserialize_i64(DurationVisitor)
}

pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer
{
    serializer.serialize_i64(duration.num_milliseconds())
}

struct DurationVisitor;

impl<'de> Visitor<'de> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result
    {
        write!(formatter, "the number of milliseconds representing a duration")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> where E: Error {
        Ok(Duration::milliseconds(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> where E: Error {
        self.visit_i64(v as i64)
    }
}