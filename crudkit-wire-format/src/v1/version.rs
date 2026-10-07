use serde::de::{Error as _, Unexpected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::openapi::RefOr;
use utoipa::openapi::schema::{ObjectBuilder, Schema, Type};
use utoipa::{PartialSchema, ToSchema};

use crate::WireFormatVersion;

/// The `wire_format_version` of every version 1 body.
///
/// Serializes as the number `1`. Deserialization rejects any other value, so a version 1 parser
/// never silently accepts a body of another version.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WireFormatVersionV1;

impl WireFormatVersionV1 {
    const NUMBER: u32 = WireFormatVersion::V1.number();
}

impl Serialize for WireFormatVersionV1 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(Self::NUMBER)
    }
}

impl<'de> Deserialize<'de> for WireFormatVersionV1 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let number = u32::deserialize(deserializer)?;
        if number == Self::NUMBER {
            Ok(Self)
        } else {
            Err(D::Error::invalid_value(
                Unexpected::Unsigned(u64::from(number)),
                &"wire format version 1",
            ))
        }
    }
}

impl PartialSchema for WireFormatVersionV1 {
    fn schema() -> RefOr<Schema> {
        ObjectBuilder::new()
            .schema_type(Type::Integer)
            .enum_values(Some([Self::NUMBER]))
            .description(Some("Wire format version of the body. Always 1."))
            .into()
    }
}

impl ToSchema for WireFormatVersionV1 {}
