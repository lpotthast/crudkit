//! The bridge between one resource's concrete model types and their erased forms.

use crate::model::{
    DynCreateField, DynCreateModel, DynReadField, DynReadModel, DynUpdateField, DynUpdateModel,
    ErasedCreateField, ErasedCreateModel, ErasedReadField, ErasedReadModel, ErasedUpdateField,
    ErasedUpdateModel, FieldAccess, Model,
};
use crudkit_core::Value;

/// Resource-specific operations on the erased models of one resource.
///
/// The handler closes over the resource's concrete create, read, and update model types, so code
/// that only holds erased models, such as a mounted instance or the dynamic data provider, stays
/// non-generic. It can only be built by [`Self::new`], which ties every operation to the same
/// three types.
#[derive(Debug, Clone, Copy)]
pub struct ModelHandler {
    deserialize_read_model: fn(serde_json::Value) -> Result<DynReadModel, serde_json::Error>,
    deserialize_update_model: fn(serde_json::Value) -> Result<DynUpdateModel, serde_json::Error>,
    read_model_to_update_model: fn(DynReadModel) -> DynUpdateModel,
    create_model_values: fn(&DynCreateModel) -> Vec<(DynCreateField, Value)>,
    read_model_values: fn(&DynReadModel) -> Vec<(DynReadField, Value)>,
    update_model_values: fn(&DynUpdateModel) -> Vec<(DynUpdateField, Value)>,
    create_model_field: fn(&str) -> Option<DynCreateField>,
    default_create_model: fn() -> DynCreateModel,
}

impl ModelHandler {
    /// Creates a handler for the concrete models `Create`, `Read`, and `Update`.
    #[must_use]
    pub fn new<Create, Read, Update>() -> Self
    where
        Create: ErasedCreateModel + Model + Default,
        Read: ErasedReadModel + Model,
        Update: ErasedUpdateModel + Model + From<Read>,
        <Create as Model>::Field: ErasedCreateField,
        <Read as Model>::Field: ErasedReadField,
        <Update as Model>::Field: ErasedUpdateField,
    {
        Self {
            deserialize_read_model: |json| {
                Ok(DynReadModel::from(serde_json::from_value::<Read>(json)?))
            },
            deserialize_update_model: |json| {
                Ok(DynUpdateModel::from(serde_json::from_value::<Update>(
                    json,
                )?))
            },
            read_model_to_update_model: |read_model| {
                DynUpdateModel::from(Update::from(read_model.downcast::<Read>()))
            },
            create_model_values: |model| field_values(model.downcast_ref::<Create>()),
            read_model_values: |model| field_values(model.downcast_ref::<Read>()),
            update_model_values: |model| field_values(model.downcast_ref::<Update>()),
            create_model_field: |name| Create::field(name).map(DynCreateField::from),
            default_create_model: || DynCreateModel::from(Create::default()),
        }
    }

    /// Deserializes the JSON of one read model.
    ///
    /// # Errors
    ///
    /// Returns the deserialization error if `json` is not a read model of this resource.
    pub fn deserialize_read_model(
        &self,
        json: serde_json::Value,
    ) -> Result<DynReadModel, serde_json::Error> {
        (self.deserialize_read_model)(json)
    }

    /// Deserializes the JSON of one update model.
    ///
    /// # Errors
    ///
    /// Returns the deserialization error if `json` is not an update model of this resource.
    pub fn deserialize_update_model(
        &self,
        json: serde_json::Value,
    ) -> Result<DynUpdateModel, serde_json::Error> {
        (self.deserialize_update_model)(json)
    }

    /// Converts a read model into the update model that edits it.
    ///
    /// # Panics
    ///
    /// Panics if `read_model` is not a read model of this resource.
    #[must_use]
    pub fn read_model_to_update_model(&self, read_model: DynReadModel) -> DynUpdateModel {
        (self.read_model_to_update_model)(read_model)
    }

    /// Returns every field of a create model together with its current value.
    ///
    /// # Panics
    ///
    /// Panics if `model` is not a create model of this resource.
    #[must_use]
    pub fn create_model_values(&self, model: &DynCreateModel) -> Vec<(DynCreateField, Value)> {
        (self.create_model_values)(model)
    }

    /// Returns every field of a read model together with its current value.
    ///
    /// # Panics
    ///
    /// Panics if `model` is not a read model of this resource.
    #[must_use]
    pub fn read_model_values(&self, model: &DynReadModel) -> Vec<(DynReadField, Value)> {
        (self.read_model_values)(model)
    }

    /// Returns every field of an update model together with its current value.
    ///
    /// # Panics
    ///
    /// Panics if `model` is not an update model of this resource.
    #[must_use]
    pub fn update_model_values(&self, model: &DynUpdateModel) -> Vec<(DynUpdateField, Value)> {
        (self.update_model_values)(model)
    }

    /// Looks up a create-model field by name. Returns `None` for unknown names.
    #[must_use]
    pub fn create_model_field(&self, name: &str) -> Option<DynCreateField> {
        (self.create_model_field)(name)
    }

    /// Returns the create model's `Default` value, the starting point of create forms.
    #[must_use]
    pub fn default_create_model(&self) -> DynCreateModel {
        (self.default_create_model)()
    }
}

/// Returns every field of `model` with its value, as the erased field type `F`.
fn field_values<M: Model, F: From<M::Field>>(model: &M) -> Vec<(F, Value)> {
    M::all_fields()
        .into_iter()
        .map(|field| {
            let value = FieldAccess::value(&field, model);
            (F::from(field), value)
        })
        .collect()
}
