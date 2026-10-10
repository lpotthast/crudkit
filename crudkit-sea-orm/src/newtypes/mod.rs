use sea_orm::{
    TryGetError, TryGetable,
    entity::prelude::*,
    sea_query::{Nullable, ValueType},
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use utoipa::openapi::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeDuration(pub time::Duration);

impl From<time::Duration> for TimeDuration {
    fn from(d: time::Duration) -> Self {
        Self(d)
    }
}

impl From<TimeDuration> for Value {
    fn from(d: TimeDuration) -> Self {
        // `Value` conversions are infallible, so durations beyond the range of `i64` microseconds
        // (about 292,000 years) saturate.
        let micros = d.0.whole_microseconds();
        let micros = <i64 as TryFrom<i128>>::try_from(micros).unwrap_or(if micros < 0 {
            i64::MIN
        } else {
            i64::MAX
        });
        Value::BigInt(Some(micros))
    }
}

impl TryFrom<Value> for TimeDuration {
    type Error = DbErr;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::BigInt(Some(us)) => Ok(TimeDuration(time::Duration::microseconds(us))),
            _ => Err(DbErr::Type("Expected BigInt for Duration".to_owned())),
        }
    }
}

impl TryGetable for TimeDuration {
    fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        let val: Option<i64> = res.try_get_by(idx).map_err(TryGetError::DbErr)?;
        match val {
            Some(us) => Ok(TimeDuration(time::Duration::microseconds(us))),
            None => Err(TryGetError::Null(format!("{idx:?}"))),
        }
    }
}

impl ValueType for TimeDuration {
    fn try_from(v: Value) -> Result<Self, sea_query::ValueTypeErr> {
        <Self as TryFrom<Value>>::try_from(v).map_err(|_| sea_query::ValueTypeErr)
    }

    fn type_name() -> String {
        "TimeDuration".to_owned()
    }

    fn array_type() -> sea_query::ArrayType {
        sea_query::ArrayType::BigInt
    }

    fn column_type() -> ColumnType {
        ColumnType::BigInteger
    }
}

impl Nullable for TimeDuration {
    fn null() -> Value {
        Value::BigInt(None)
    }
}

/// Serialize as microseconds (i64)
impl Serialize for TimeDuration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let micros =
            <i64 as TryFrom<i128>>::try_from(self.0.whole_microseconds()).map_err(|_| {
                serde::ser::Error::custom("duration exceeds the range of i64 microseconds")
            })?;
        serializer.serialize_i64(micros)
    }
}

/// Deserialize from microseconds (i64)
impl<'de> Deserialize<'de> for TimeDuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let microseconds = i64::deserialize(deserializer)?;
        Ok(TimeDuration(time::Duration::microseconds(microseconds)))
    }
}

impl utoipa::ToSchema for TimeDuration {
    fn name() -> Cow<'static, str> {
        Cow::Borrowed("TimeDuration")
    }
}

impl utoipa::PartialSchema for TimeDuration {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::SchemaType::Type(Type::Integer))
            .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                utoipa::openapi::KnownFormat::Int64,
            )))
            .description(Some("Duration in microseconds"))
            .into()
    }
}
