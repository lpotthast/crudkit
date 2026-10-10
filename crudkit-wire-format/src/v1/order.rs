use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Sort direction of one ordering criterion.
///
/// Serializes as `"asc"` or `"desc"`. Deserialization also accepts `"ascending"`, `"Asc"`,
/// `"descending"`, and `"Desc"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum OrderV1 {
    #[serde(rename(
        serialize = "asc",
        deserialize = "asc",
        deserialize = "ascending",
        deserialize = "Asc"
    ))]
    Asc,
    #[serde(rename(
        serialize = "desc",
        deserialize = "desc",
        deserialize = "descending",
        deserialize = "Desc"
    ))]
    Desc,
}
