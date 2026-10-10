use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Value of one component of an entity ID.
///
/// Date-times use explicit string formats, independent of the Cargo features of the `time` crate:
/// `PrimitiveDateTime` as `YYYY-MM-DDTHH:MM:SS[.fraction]` and `OffsetDateTime` as RFC 3339.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum IdValueV1 {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Bool(bool),
    String(String),
    Uuid(uuid::Uuid),
    PrimitiveDateTime(#[serde(with = "primitive_date_time")] time::PrimitiveDateTime),
    OffsetDateTime(#[serde(with = "time::serde::rfc3339")] time::OffsetDateTime),
}

/// Serializes a `PrimitiveDateTime` as `YYYY-MM-DDTHH:MM:SS`, followed by a fraction of a second only
/// when it is not zero.
mod primitive_date_time {
    use serde::{Deserialize, Deserializer, Serializer};
    use time::PrimitiveDateTime;
    use time::format_description::BorrowedFormatItem;
    use time::macros::format_description;

    const WHOLE_SECONDS: &[BorrowedFormatItem<'_>] =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");
    const FRACTIONAL_SECONDS: &[BorrowedFormatItem<'_>] =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]");

    pub(super) fn serialize<S: Serializer>(
        value: &PrimitiveDateTime,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let format = if value.nanosecond() == 0 {
            WHOLE_SECONDS
        } else {
            FRACTIONAL_SECONDS
        };
        let formatted = value.format(format).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&formatted)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<PrimitiveDateTime, D::Error> {
        let text = String::deserialize(deserializer)?;
        PrimitiveDateTime::parse(&text, FRACTIONAL_SECONDS)
            .or_else(|_| PrimitiveDateTime::parse(&text, WHOLE_SECONDS))
            .map_err(serde::de::Error::custom)
    }
}

/// An entity ID: its components in order. It does not name the entity's resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[schema(value_type = Vec<Object>)]
pub struct SerializableIdV1(pub Vec<SerializableIdEntryV1>);

/// One component of an entity ID. Serialized as a `[field_name, value]` pair.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(into = "(String, IdValueV1)", from = "(String, IdValueV1)")]
pub struct SerializableIdEntryV1 {
    pub field_name: String,
    pub value: IdValueV1,
}

impl From<SerializableIdEntryV1> for (String, IdValueV1) {
    fn from(entry: SerializableIdEntryV1) -> Self {
        (entry.field_name, entry.value)
    }
}

impl From<(String, IdValueV1)> for SerializableIdEntryV1 {
    fn from((field_name, value): (String, IdValueV1)) -> Self {
        Self { field_name, value }
    }
}
