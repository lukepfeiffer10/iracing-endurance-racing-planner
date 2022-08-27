use barrel::types;
use barrel::types::Type;

pub fn datetime_with_timezone() -> Type {
    types::custom("timestamptz")
}

pub fn interval() -> Type {
    types::custom("interval")
}

pub fn smallint() -> Type {
    types::custom("smallint")
}

pub fn real() -> Type {
    types::custom("real")
}
