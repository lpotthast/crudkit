//! Serializable descriptions of CrudKit views.

#![deny(missing_docs)]

use crudkit_core::id::SerializableId;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Stable name of CrudKit's built-in table view.
pub const TABLE_VIEW: &str = "crudkit.table";
/// Stable name of CrudKit's built-in create view.
pub const CREATE_VIEW: &str = "crudkit.create";
/// Stable name of CrudKit's built-in read view.
pub const READ_VIEW: &str = "crudkit.read";
/// Stable name of CrudKit's built-in edit view.
pub const EDIT_VIEW: &str = "crudkit.edit";

/// An open, serializable description of a view shown by a CrudKit instance.
///
/// CrudKit supplies constructors for its built-in views, but applications may
/// use any stable name and JSON payload for views registered in a
/// `CrudViewRegistry`. Entity-oriented custom views can set [`Self::subject`]
/// so nested instances can resolve their parent resource without knowing the
/// custom view name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrudView {
    /// Stable registry key that selects the renderer.
    pub name: String,
    /// Renderer-specific data, or JSON `null` when the view has no payload.
    #[serde(default)]
    pub payload: serde_json::Value,
    /// Entity represented by the view, when parent-resource composition needs one. Serialized in
    /// the explicit `SerializableIdV1` format.
    #[serde(default, skip_serializing_if = "Option::is_none", with = "subject_v1")]
    pub subject: Option<SerializableId>,
}

/// Serializes a view's subject in the version 1 wire form of IDs.
mod subject_v1 {
    use crudkit_core::id::SerializableId;
    use crudkit_wire_format::v1::SerializableIdV1;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    // Serde's `with` passes the field by reference.
    #[allow(clippy::ref_option)]
    pub(super) fn serialize<S: Serializer>(
        subject: &Option<SerializableId>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        subject
            .clone()
            .map(SerializableIdV1::from)
            .serialize(serializer)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<SerializableId>, D::Error> {
        Ok(Option::<SerializableIdV1>::deserialize(deserializer)?.map(SerializableId::from))
    }
}

impl CrudView {
    /// Describes an application-defined view with an empty payload.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            payload: serde_json::Value::Null,
            subject: None,
        }
    }

    /// Sets an already serialized JSON payload.
    #[must_use]
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }

    /// Serializes and sets a typed payload.
    ///
    /// # Errors
    ///
    /// Returns the serialization error produced by `serde_json` when `payload`
    /// cannot be represented as JSON.
    pub fn with_typed_payload<T: Serialize>(
        mut self,
        payload: T,
    ) -> Result<Self, serde_json::Error> {
        self.payload = serde_json::to_value(payload)?;
        Ok(self)
    }

    /// Deserializes this view's payload into a concrete type.
    ///
    /// # Errors
    ///
    /// Returns the deserialization error produced by `serde_json` when the
    /// payload does not match `T`.
    pub fn typed_payload<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.payload.clone())
    }

    /// Marks the entity represented by this view.
    #[must_use]
    pub fn with_subject(mut self, subject: SerializableId) -> Self {
        self.subject = Some(subject);
        self
    }

    /// Describes CrudKit's built-in table view.
    #[must_use]
    pub fn table() -> Self {
        Self::new(TABLE_VIEW)
    }

    /// Describes CrudKit's built-in create view.
    #[must_use]
    pub fn create() -> Self {
        Self::new(CREATE_VIEW)
    }

    /// Describes CrudKit's built-in read view for `id`.
    #[must_use]
    pub fn read(id: SerializableId) -> Self {
        Self::new(READ_VIEW).with_subject(id)
    }

    /// Describes CrudKit's built-in edit view for `id`.
    #[must_use]
    pub fn edit(id: SerializableId) -> Self {
        Self::new(EDIT_VIEW).with_subject(id)
    }
}

impl Default for CrudView {
    fn default() -> Self {
        Self::table()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;
    use crudkit_core::id::{IdValue, SerializableIdEntry};
    use serde::{Deserialize, Serialize};

    fn id() -> SerializableId {
        SerializableId(vec![SerializableIdEntry {
            field_name: "id".to_owned(),
            value: IdValue::I64(42),
        }])
    }

    #[test]
    fn subject_serializes_as_a_version_1_id() {
        let json = serde_json::to_value(CrudView::read(id())).expect("view should serialize");
        assert_that!(json["subject"].clone())
            .is_equal_to(serde_json::json!([["id", { "I64": 42 }]]));
    }

    #[test]
    fn json_and_subject_round_trip() {
        let view = CrudView::new("example.detail")
            .with_payload(serde_json::json!({ "tab": "activity" }))
            .with_subject(id());

        let json = serde_json::to_string(&view).expect("view should serialize");
        let restored: CrudView = serde_json::from_str(&json).expect("view should deserialize");

        assert_that!(restored).is_equal_to(view);
    }

    #[test]
    fn typed_payload_round_trip() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct Payload {
            tab: String,
        }

        let view = CrudView::new("example.detail")
            .with_typed_payload(Payload {
                tab: "activity".to_owned(),
            })
            .expect("payload should serialize");

        assert_that!(
            view.typed_payload::<Payload>()
                .expect("payload should deserialize")
        )
        .is_equal_to(Payload {
            tab: "activity".to_owned(),
        });
    }

    #[test]
    fn built_in_constructors_are_stable() {
        assert_that!(CrudView::table()).is_equal_to(CrudView::new(TABLE_VIEW));
        assert_that!(CrudView::create()).is_equal_to(CrudView::new(CREATE_VIEW));
        assert_that!(CrudView::read(id()).name).is_equal_to(READ_VIEW.to_owned());
        assert_that!(CrudView::read(id()).subject).is_equal_to(Some(id()));
        assert_that!(CrudView::edit(id()).name).is_equal_to(EDIT_VIEW.to_owned());
        assert_that!(CrudView::edit(id()).subject).is_equal_to(Some(id()));
    }
}
