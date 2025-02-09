#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use async_trait::async_trait;
use crudkit_condition::ConditionClauseValue;
use crudkit_id::SerializableId;
use dyn_clone::DynClone;
use dyn_hash::DynHash;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::Arc;
use std::{
    any::Any,
    borrow::Cow,
    fmt::{Debug, Display},
    hash::Hash,
};
use time::format_description::well_known::Rfc3339;
use tracing::warn;

pub mod crud_rest_data_provider;
pub mod crud_rest_data_provider_dyn;
pub mod error;
pub mod files;
pub mod request_error;
pub mod requests;
pub mod requests_dyn;
pub mod reqwest_executor;

/*
* Reexport common modules.
* This allows the user to only
*
* - `use crudkit_web::prelude::*` and
* - derive all common proc macros
*
* without the need to add more use declaration or
* to manually depend on other crud crates such as "crudkit_id",
* which are required for many derive macro implementations.
*/
use crate::request_error::RequestError;
pub use crudkit_condition;
pub use crudkit_id;
pub use crudkit_shared;
pub use crudkit_validation;
pub use crudkit_websocket;
use dyn_eq::DynEq;

pub mod prelude {
    pub use crudkit_condition;
    pub use crudkit_id;
    pub use crudkit_shared;
    pub use crudkit_validation;
    pub use crudkit_websocket;

    pub use derive_crud_action_payload::CkActionPayload;
    pub use derive_crud_resource::CkResource;
    pub use derive_crud_selectable::CkSelectable;
    pub use derive_crudkit_id::CkId;
    pub use derive_field::CkField;
    pub use derive_field_value::CkFieldValue;

    pub use super::crud_rest_data_provider::CreateOne;
    pub use super::crud_rest_data_provider::CrudRestDataProvider;
    pub use super::crud_rest_data_provider::DeleteById;
    pub use super::crud_rest_data_provider::ReadCount;
    pub use super::crud_rest_data_provider::ReadMany;
    pub use super::crud_rest_data_provider::ReadOne;
    pub use super::crud_rest_data_provider::UpdateOne;
    pub use super::error::ErrorInfo;
    pub use super::files::FileResource;
    pub use super::files::ListFileError;
    pub use super::files::ListFilesResponse;
    pub use super::request_error::RequestError;
    // TODO: add as static and dynamic modules
    //pub use super::requests::request;
    //pub use super::requests::request_delete;
    //pub use super::requests::request_get;
    //pub use super::requests::request_post;
    //pub use super::requests::request_put;
    pub use super::reqwest_executor::ReqwestExecutor;
    pub use super::CrudActionPayload;
    pub use super::CrudDataTrait;
    pub use super::CrudField;
    pub use super::CrudFieldNameTrait;
    pub use super::CrudFieldValueTrait;
    pub use super::CrudIdTrait;
    pub use super::CrudMainTrait;
    pub use super::CrudModel;
    pub use super::CrudResourceTrait;
    pub use super::CrudSelectableSource;
    pub use super::CrudSelectableTrait;
    pub use super::CrudSimpleView;
    pub use super::CrudView;
    pub use super::DeletableModel;
    pub use super::Elem;
    pub use super::EmptyActionPayload;
    pub use super::Enclosing;
    pub use super::FieldMode;
    pub use super::FieldOptions;
    pub use super::Group;
    pub use super::HeaderOptions;
    pub use super::Label;
    pub use super::Layout;
    pub use super::NoData;
    pub use super::OrderByUpdateOptions;
    pub use super::SerializableCrudView;
    pub use super::Tab;
    pub use super::Value;
    pub use super::Variant;
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoData {
    NotYetLoaded,
    FetchFailed(RequestError),
    // TODO: Can probably be deleted at some point...
    FetchReturnedNothing,
    CreateFailed(RequestError),
    // TODO: Can probably be deleted at some point...
    CreateReturnedNothing,
    UpdateFailed(RequestError),
    // TODO: Can probably be deleted at some point...
    UpdateReturnedNothing,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Variant {
    Default, // TODO: Consider to remove this variant...
    Primary,
    Secondary,
    Success,
    Info,
    Warn,
    Danger,
}

impl Variant {
    pub fn class_name(&self) -> &'static str {
        match self {
            Variant::Default => "type-default",
            Variant::Primary => "type-primary",
            Variant::Secondary => "type-secondary",
            Variant::Success => "type-success",
            Variant::Info => "type-default",
            Variant::Warn => "type-warn",
            Variant::Danger => "type-danger",
        }
    }
}

// TODO: This con be computed statically!
//impl From<Variant> for Classes {
//    fn from(variant: Variant) -> Self {
//        match variant {
//            Variant::Default => classes!("type-default"),
//            Variant::Primary => classes!("type-primary"),
//            Variant::Secondary => classes!("type-secondary"),
//            Variant::Success => classes!("type-success"),
//            Variant::Info => classes!("type-default"),
//            Variant::Warn => classes!("type-warn"),
//            Variant::Danger => classes!("type-danger"),
//        }
//    }
//}

/// Any possible model.
#[derive(Debug, Clone, PartialEq)]
pub enum CrudModel<T: CrudMainTrait> {
    ReadModel(T::ReadModel),
    CreateModel(T::CreateModel),
    UpdateModel(T::UpdateModel),
}

/// Any possible field.
#[derive(Debug, Clone, PartialEq)]
pub enum CrudField<T: CrudMainTrait> {
    ReadModel(<T::ReadModel as CrudDataTrait>::Field),
    CreateModel(<T::CreateModel as CrudDataTrait>::Field),
    UpdateModel(<T::UpdateModel as CrudDataTrait>::Field),
}

// TODO: impl Clone if both types are clone, same for debug, ...
pub trait CrudMainTrait:
    CrudResourceTrait + PartialEq + Default + Debug + Clone + Serialize + Send + Sync
{
    type CreateModel: CrudDataTrait + Default + Send + Sync;

    type ReadModelIdField: crudkit_id::IdField + Serialize + Send + Sync;
    type ReadModelId: Serialize
        + DeserializeOwned
        + crudkit_id::Id<Field = Self::ReadModelIdField>
        + PartialEq
        + Clone
        + Send
        + Sync;
    type ReadModel: Serialize
        + CrudDataTrait
        + Into<Self::UpdateModel>
        + CrudIdTrait<Id = Self::ReadModelId>
        + Send
        + Sync;

    type UpdateModelIdField: crudkit_id::IdField + Serialize + Send + Sync;
    type UpdateModelId: Serialize
        + DeserializeOwned
        + crudkit_id::Id<Field = Self::UpdateModelIdField>
        + PartialEq
        + Clone
        + Send
        + Sync;
    type UpdateModel: Serialize
        + CrudDataTrait
        + CrudIdTrait<Id = Self::UpdateModelId>
        + Send
        + Sync;

    type ActionPayload: Serialize + CrudActionPayload + Send + Sync;
}

/// Marker trait for specifying data which can be used as payload for CRUD actions.
/// Use the `CrudActionPayload` derive macro from derive-crud-action-payload to implement this for your type.
pub trait CrudActionPayload:
    PartialEq + Clone + Debug + Serialize + DeserializeOwned + Send + Sync
{
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct EmptyActionPayload {}

impl CrudActionPayload for EmptyActionPayload {}

// Note: This does not have CrudIdTrait as a super trait, as not all data model (namely the CreateModel) can supply an ID!
pub trait CrudDataTrait:
    PartialEq + Clone + Debug + Serialize + DeserializeOwned + Send + Sync
{
    type Field: CrudFieldNameTrait
        + CrudFieldValueTrait<Self>
        + PartialEq
        + Eq
        + Hash
        + Clone
        + Debug
        + Serialize
        + DeserializeOwned
        + Send
        + Sync;

    fn get_all_fields() -> Vec<Self::Field>;
    fn get_field(field_name: &str) -> Self::Field;
}

/// Allows us to access the ID of an entity.
/// The ID type must provide more fine-grained access (for example to individual fields).
pub trait CrudIdTrait {
    type Id: crudkit_id::Id;

    fn get_id(&self) -> Self::Id;
}

impl<T: CrudIdTrait> CrudIdTrait for &T {
    type Id = T::Id;

    fn get_id(&self) -> Self::Id {
        T::get_id(&self)
    }
}

/// Anything that has an identifier in form of a `SerializableId`.
///
/// Trait is expected to be object safe.
pub trait Identifiable: Debug + DynClone + DynEq + Send + Sync {
    fn get_id(&self) -> SerializableId;
}
dyn_eq::eq_trait_object!(Identifiable);
dyn_clone::clone_trait_object!(Identifiable);

pub type AnyIdentifiable = Arc<dyn Identifiable>;

pub trait NamedProperty: Send + Sync {
    fn get_name(&self) -> String;
}

pub trait DynSerialize: Send + Sync {
    fn serialize(&self) -> serde_json::Value;
}

#[typetag::serde]
pub trait Field: Debug + NamedProperty + DynClone + DynEq + DynHash + Send + Sync {
    fn set_value(&self, model: &mut AnyModel, value: Value);
}
dyn_eq::eq_trait_object!(Field);
dyn_clone::clone_trait_object!(Field);
dyn_hash::hash_trait_object!(Field);

pub type AnyField = Arc<dyn Field>;

#[typetag::serde]
pub trait Model:
    Identifiable + Debug + DynClone + DynEq + downcast_rs::Downcast + Send + Sync
{
}
dyn_eq::eq_trait_object!(Model);
dyn_clone::clone_trait_object!(Model);
downcast_rs::impl_downcast!(Model);

pub type AnyModel = Box<dyn Model>;

// TODO: Serialization?
pub trait ActionPayload: Debug + DynClone + DynEq + Send + Sync {}

pub type AnyActionPayload = Arc<dyn ActionPayload>;

pub trait CrudResourceTrait {
    fn get_resource_name() -> &'static str
    where
        Self: Sized;
}

#[async_trait]
pub trait CrudSelectableSource: Debug + Send + Sync {
    type Selectable: CrudSelectableTrait;

    async fn load(
        &self,
    ) -> Result<Vec<Self::Selectable>, Arc<dyn std::error::Error + Send + Sync + 'static>>;

    fn set_selectable(&mut self, selectable: Vec<Self::Selectable>);

    /// May return None if selectable options were not yet loaded.
    fn get_selectable(&self) -> Option<Vec<Self::Selectable>>;
}

//#[typetag::serde(tag = "type")]
pub trait CrudSelectableTrait: Debug + Display + DynClone + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}
dyn_clone::clone_trait_object!(CrudSelectableTrait);

/// All variants should be stateless / copy-replaceable.
// TODO: DEFERRED: Implement Serialize and Deserialize with typetag when wasm is supported in typetag. Comment in "typetag" occurrences.
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    OptionalString(Option<String>),
    Text(String), // TODO: Add optional text!, TODO: Remove this variant altogether and make "text" an optional editing mode for string values!
    Json(JsonValue), // TODO: Add optional json value
    OptionalJson(Option<JsonValue>),
    UuidV4(uuid::Uuid), // TODO: Add optional UuidV4 value
    UuidV7(uuid::Uuid), // TODO: Add optional UuidV7 value
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    OptionalI32(Option<i32>),
    OptionalI64(Option<i64>),
    OptionalU32(Option<u32>),
    OptionalU64(Option<u64>),
    F32(f32),
    F64(f64),
    Bool(bool),
    // Specialized bool-case, render as a green check mark if false and an orange exclamation mark if true.
    ValidationStatus(bool),
    PrimitiveDateTime(time::PrimitiveDateTime),
    OffsetDateTime(time::OffsetDateTime),
    OptionalPrimitiveDateTime(Option<time::PrimitiveDateTime>),
    OptionalOffsetDateTime(Option<time::OffsetDateTime>),
    OneToOneRelation(Option<u32>),
    Reference(Vec<Box<dyn crudkit_id::IdField>>), // TODO: This variant should probably be named "Reference". Can it carry a "SerializableId" (as it is of known size)?
    Custom(()),
    Select(Box<dyn CrudSelectableTrait>),
    Multiselect(Vec<Box<dyn CrudSelectableTrait>>),
    OptionalSelect(Option<Box<dyn CrudSelectableTrait>>),
    OptionalMultiselect(Option<Vec<Box<dyn CrudSelectableTrait>>>),
    //Select(Box<dyn CrudSelectableSource<Selectable = dyn CrudSelectableTrait>>),
}

#[derive(Debug, Clone)]
pub struct JsonValue {
    value: serde_json::Value,
    string_representation: String,
}

impl JsonValue {
    pub fn new(value: serde_json::Value) -> Self {
        let string_representation = serde_json::to_string(&value).unwrap();
        Self {
            value,
            string_representation,
        }
    }

    pub fn set_value(&mut self, value: serde_json::Value) {
        self.value = value;
        self.string_representation = serde_json::to_string(&self.value).unwrap();
    }

    pub fn get_value(&self) -> &serde_json::Value {
        &self.value
    }

    pub fn get_string_representation(&self) -> &str {
        self.string_representation.as_str()
    }
}

impl Into<serde_json::Value> for JsonValue {
    fn into(self) -> serde_json::Value {
        self.value
    }
}

impl Into<String> for JsonValue {
    fn into(self) -> String {
        self.string_representation
    }
}

impl Into<Value> for crudkit_shared::Value {
    fn into(self) -> Value {
        match self {
            crudkit_shared::Value::String(value) => Value::String(value), // TODO: How can we differentiate between String and Text?
            crudkit_shared::Value::Json(value) => Value::Json(JsonValue::new(value)),
            crudkit_shared::Value::UuidV4(value) => Value::UuidV4(value),
            crudkit_shared::Value::UuidV7(value) => Value::UuidV7(value),
            crudkit_shared::Value::I32(value) => Value::I32(value),
            crudkit_shared::Value::I64(value) => Value::I64(value),
            crudkit_shared::Value::U8Vec(_values) => todo!("support vector types"),
            crudkit_shared::Value::I32Vec(_values) => todo!("support vector types"),
            crudkit_shared::Value::I64Vec(_values) => todo!("support vector types"),
            crudkit_shared::Value::U32(value) => Value::U32(value),
            crudkit_shared::Value::U64(value) => Value::U64(value),
            crudkit_shared::Value::F32(value) => Value::F32(value),
            crudkit_shared::Value::F64(value) => Value::F64(value),
            crudkit_shared::Value::Bool(value) => Value::Bool(value),
            crudkit_shared::Value::PrimitiveDateTime(value) => Value::PrimitiveDateTime(value),
            crudkit_shared::Value::OffsetDateTime(value) => Value::OffsetDateTime(value),
        }
    }
}

impl Into<Value> for crudkit_id::IdValue {
    fn into(self) -> Value {
        match self {
            crudkit_id::IdValue::String(value) => Value::String(value), // TODO: How can we differentiate between String and Text?
            crudkit_id::IdValue::UuidV4(value) => Value::UuidV4(value),
            crudkit_id::IdValue::UuidV7(value) => Value::UuidV7(value),
            crudkit_id::IdValue::I32(value) => Value::I32(value),
            crudkit_id::IdValue::I64(value) => Value::I64(value),
            crudkit_id::IdValue::U32(value) => Value::U32(value),
            crudkit_id::IdValue::U64(value) => Value::U64(value),
            crudkit_id::IdValue::Bool(value) => Value::Bool(value),
            crudkit_id::IdValue::PrimitiveDateTime(value) => Value::PrimitiveDateTime(value),
            crudkit_id::IdValue::OffsetDateTime(value) => Value::OffsetDateTime(value),
        }
    }
}

impl Value {
    pub fn take_string(self) -> String {
        match self {
            Self::String(string) => string,
            Self::Text(string) => string,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_string(self) -> Option<String> {
        match self {
            Self::OptionalString(string) => string,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_text(self) -> String {
        match self {
            Self::Text(string) => string,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_json_value(self) -> JsonValue {
        match self {
            Self::Json(json) => json,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_inner_json_value(self) -> serde_json::Value {
        match self {
            Self::Json(json) => json.into(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_json_value(self) -> Option<JsonValue> {
        match self {
            Self::OptionalJson(json) => json,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_uuid_v4(self) -> uuid::Uuid {
        match self {
            Self::UuidV4(uuid) => uuid,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_uuid_v7(self) -> uuid::Uuid {
        match self {
            Self::UuidV7(uuid) => uuid,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_u32(self) -> u32 {
        match self {
            Self::U32(u32) => u32,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_u64(self) -> u64 {
        match self {
            Self::U64(u64) => u64,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_u32_or_parse(self) -> u32 {
        match self {
            Self::U32(u32) => u32,
            Self::String(string) => string.parse().unwrap(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_u32(self) -> Option<u32> {
        match self {
            Self::OptionalU32(u32) => u32,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_i32(self) -> i32 {
        match self {
            Self::I32(i32) => i32,
            // This has some potential data loss...
            // TODO: Can we remove this? Without, this created a panic in agnite/servers/labels/new
            Self::U32(u32) => u32 as i32,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_i64(self) -> i64 {
        match self {
            Self::I64(i64) => i64,
            // This has some potential data loss...
            // TODO: Can we remove this? Without, this created a panic in agnite/servers/labels/new
            //Self::U32(u32) => u32 as i32,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_i32(self) -> Option<i32> {
        match self {
            Self::I32(value) => Some(value),
            Self::OptionalI32(value) => value,
            Self::String(string) => string
                .parse::<i32>()
                .map_err(|err| warn!("take_optional_i32 could not pase string: {err}"))
                .ok(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_i64(self) -> Option<i64> {
        match self {
            Self::I64(value) => Some(value),
            Self::OptionalI64(value) => value,
            Self::String(string) => string
                .parse::<i64>()
                .map_err(|err| warn!("take_optional_i64 could not pase string: {err}"))
                .ok(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_u64(self) -> Option<u64> {
        match self {
            Self::U64(value) => Some(value),
            Self::OptionalU64(value) => value,
            Self::String(value) => value
                .parse::<u64>()
                .map_err(|err| warn!("take_optional_u64 could not pase string: {err}"))
                .ok(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_f32(self) -> f32 {
        match self {
            Self::F32(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_f64(self) -> f64 {
        match self {
            Self::F64(value) => value,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_bool(self) -> bool {
        match self {
            Self::Bool(bool) => bool,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    /*
    pub fn take_bool_or_parse(self) -> bool {
        match self {
            Self::Bool(bool) => bool,
            Self::String(string) => string.parse().unwrap(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    */
    pub fn take_validation_status(self) -> bool {
        match self {
            Self::ValidationStatus(bool) => bool,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_primitive_date_time(self) -> time::PrimitiveDateTime {
        match self {
            Self::PrimitiveDateTime(primitive_date_time) => primitive_date_time,
            Self::String(string) => time::PrimitiveDateTime::parse(&string, &Rfc3339).unwrap(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_offset_date_time(self) -> time::OffsetDateTime {
        match self {
            Self::OffsetDateTime(offset_date_time) => offset_date_time,
            Self::String(string) => time::OffsetDateTime::parse(&string, &Rfc3339).unwrap(),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_primitive_date_time(self) -> Option<time::PrimitiveDateTime> {
        match self {
            Self::PrimitiveDateTime(primitive_date_time) => Some(primitive_date_time),
            Self::OptionalPrimitiveDateTime(optional_primitive_date_time) => {
                optional_primitive_date_time
            }
            // TODO: We might want to catch parsing errors and return an empty optional here.
            Self::String(string) => {
                Some(time::PrimitiveDateTime::parse(&string, &Rfc3339).unwrap())
            }
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_optional_offset_date_time(self) -> Option<time::OffsetDateTime> {
        match self {
            Self::OffsetDateTime(offset_date_time) => Some(offset_date_time),
            Self::OptionalOffsetDateTime(optional_offset_date_time) => optional_offset_date_time,
            // TODO: We might want to catch parsing errors and return an empty optional here.
            Self::String(string) => Some(time::OffsetDateTime::parse(&string, &Rfc3339).unwrap()),
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_select(self) -> Box<dyn CrudSelectableTrait> {
        match self {
            Self::Select(selected) => selected,
            other => panic!("unsupported type, expected select, found: {other:?}"),
        }
    }
    pub fn take_select_downcast_to<T: Clone + 'static>(self) -> T {
        match self {
            Self::Select(selected) => selected.as_any().downcast_ref::<T>().unwrap().clone(),
            other => panic!("Expected variant `Value::Select` but got `{other:?}`."),
        }
    }
    pub fn take_optional_select_downcast_to<T: Clone + 'static>(self) -> Option<T> {
        match self {
            Self::OptionalSelect(selected) => {
                selected.map(|it| it.as_any().downcast_ref::<T>().unwrap().clone())
            }
            other => panic!("Expected variant `Value::OptionalSelect` but got `{other:?}`."),
        }
    }
    pub fn take_multiselect(self) -> Vec<Box<dyn CrudSelectableTrait>> {
        match self {
            Self::Multiselect(selected) => selected,
            other => panic!("unsupported type, expected select, found: {other:?}"),
        }
    }
    pub fn take_multiselect_downcast_to<T: Clone + 'static>(self) -> Vec<T> {
        match self {
            Self::Multiselect(selected) => selected
                .into_iter()
                .map(|value| value.as_any().downcast_ref::<T>().unwrap().clone())
                .collect(),
            _ => panic!("unsupported type provided"),
        }
    }
    pub fn take_optional_multiselect_downcast_to<T: Clone + 'static>(self) -> Option<Vec<T>> {
        match self {
            Self::OptionalMultiselect(selected) => selected.map(|it| {
                it.into_iter()
                    .map(|it| it.as_any().downcast_ref::<T>().unwrap().clone())
                    .collect()
            }),
            _ => panic!("unsupported type provided"),
        }
    }
    pub fn take_one_to_one_relation(self) -> Option<u32> {
        match self {
            Value::U32(u32) => Some(u32),
            Value::OptionalU32(optional_u32) => optional_u32,
            Value::OneToOneRelation(optional_u32) => optional_u32,
            other => panic!("Expected Value of variant 'U32', 'OptionalU32' or 'OneToOneRelation'. Received: {other:?}"),
        }
    }
    pub fn take_reference(self) -> Vec<Box<dyn crudkit_id::IdField>> {
        match self {
            Value::Reference(id) => id,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
    pub fn take_custom(self) -> () {
        match self {
            Value::Custom(nothing) => nothing,
            other => panic!("unsupported type provided: {other:?} "),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(value) => f.write_str(value),
            Value::OptionalString(value) => match value {
                Some(value) => f.write_str(value),
                None => f.write_str("-"),
            },
            Value::Text(value) => f.write_str(value),
            Value::Json(value) => f.write_str(value.get_string_representation()),
            Value::OptionalJson(value) => match value {
                Some(value) => f.write_str(value.get_string_representation()),
                None => f.write_str("-"),
            },
            Value::UuidV4(value) => f.write_str(&value.to_string()),
            Value::UuidV7(value) => f.write_str(&value.to_string()),
            Value::I32(value) => f.write_str(&value.to_string()),
            Value::I64(value) => f.write_str(&value.to_string()),
            Value::U32(value) => f.write_str(&value.to_string()),
            Value::U64(value) => f.write_str(&value.to_string()),
            Value::OptionalI32(value) => match value {
                Some(value) => f.write_str(&value.to_string()),
                None => f.write_str("-"),
            },
            Value::OptionalI64(value) => match value {
                Some(value) => f.write_str(&value.to_string()),
                None => f.write_str("-"),
            },
            Value::OptionalU32(value) => match value {
                Some(value) => f.write_str(&value.to_string()),
                None => f.write_str("-"),
            },
            Value::OptionalU64(value) => match value {
                Some(value) => f.write_str(&value.to_string()),
                None => f.write_str("-"),
            },
            Value::F32(value) => f.write_str(&value.to_string()),
            Value::F64(value) => f.write_str(&value.to_string()),
            Value::Bool(value) => f.write_str(&value.to_string()),
            Value::ValidationStatus(value) => f.write_str(&value.to_string()),
            Value::PrimitiveDateTime(value) => f.write_str(&value.format(&Rfc3339).unwrap()),
            Value::OffsetDateTime(value) => f.write_str(&value.format(&Rfc3339).unwrap()),
            Value::OptionalPrimitiveDateTime(value) => match value {
                Some(value) => f.write_str(&value.format(&Rfc3339).unwrap()),
                None => f.write_str(""),
            },
            Value::OptionalOffsetDateTime(value) => match value {
                Some(value) => f.write_str(&value.format(&Rfc3339).unwrap()),
                None => f.write_str(""),
            },
            Value::OneToOneRelation(value) => match value {
                Some(value) => f.write_str(&value.to_string()),
                None => f.write_str(""),
            },
            Value::Reference(id) => {
                for field in id {
                    f.write_fmt(format_args!(
                        "'{}': {:?}",
                        field.name(),
                        field.to_value() // was: into_dyn_value
                    ))?;
                }
                Ok(())
            }
            Value::Custom(_) => f.write_str("Custom"),
            Value::Select(selected) => f.write_str(&selected.to_string()),
            Value::OptionalSelect(selected) => match selected {
                Some(selected) => f.write_str(&selected.to_string()),
                None => f.write_str("NONE"),
            },
            Value::Multiselect(selected) => {
                for value in selected {
                    f.write_str(&value.to_string())?
                }
                Ok(())
            }
            Value::OptionalMultiselect(selected) => match selected {
                Some(selected) => {
                    for value in selected {
                        f.write_str(&value.to_string())?
                    }
                    Ok(())
                }
                None => f.write_str("NONE"),
            },
        }
    }
}

impl Into<ConditionClauseValue> for Value {
    fn into(self) -> ConditionClauseValue {
        match self {
            // TODO: Complete mapping!!
            Value::String(value) => ConditionClauseValue::String(value),
            Value::OptionalString(_value) => todo!(),
            Value::Text(value) => ConditionClauseValue::String(value),
            Value::Json(value) => ConditionClauseValue::Json(value.into()),
            Value::OptionalJson(_value) => todo!(),
            Value::UuidV4(value) => ConditionClauseValue::UuidV4(value),
            Value::UuidV7(value) => ConditionClauseValue::UuidV7(value),
            Value::I32(value) => ConditionClauseValue::I32(value),
            Value::I64(value) => ConditionClauseValue::I64(value),
            Value::U32(value) => ConditionClauseValue::U32(value),
            Value::U64(value) => ConditionClauseValue::U64(value),
            Value::OptionalI32(_value) => todo!(),
            Value::OptionalI64(_value) => todo!(),
            Value::OptionalU32(_value) => todo!(),
            Value::OptionalU64(_value) => todo!(),
            Value::F32(value) => ConditionClauseValue::F32(value),
            Value::F64(value) => ConditionClauseValue::F64(value),
            Value::Bool(value) => ConditionClauseValue::Bool(value),
            Value::ValidationStatus(_value) => todo!(),
            Value::PrimitiveDateTime(_value) => todo!(),
            Value::OffsetDateTime(_value) => todo!(),
            Value::OptionalPrimitiveDateTime(_value) => todo!(),
            Value::OptionalOffsetDateTime(_value) => todo!(),
            Value::OneToOneRelation(_value) => todo!(),
            Value::Reference(_value) => todo!(),
            Value::Custom(_value) => todo!(),
            Value::Select(_value) => todo!(),
            Value::Multiselect(_value) => todo!(),
            Value::OptionalSelect(_value) => todo!(),
            Value::OptionalMultiselect(_value) => todo!(),
        }
    }
}

// TODO: Remove
//#[typetag::serde]
/*
impl crudkit_id::IdFieldValue for Value {
    fn into_condition_clause_value(&self) -> ConditionClauseValue {
        // Note: This requires clone, because we take &self. We take &self, so that the trait remains dynamically usable.
        self.clone().into()
    }
}
*/

pub trait CrudFieldNameTrait {
    fn get_name(&self) -> &'static str;
}

pub trait CrudFieldValueTrait<T> {
    fn get_value(&self, entity: &T) -> Value;
    fn set_value(&self, entity: &mut T, value: Value);
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CrudView<ReadId, UpdateId>
where
    ReadId: crudkit_id::Id + Serialize + DeserializeOwned,
    UpdateId: crudkit_id::Id + Serialize + DeserializeOwned,
{
    List,
    Create,
    #[serde(bound = "")]
    Read(ReadId),
    #[serde(bound = "")]
    Edit(UpdateId),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SerializableCrudView {
    #[default]
    List,
    Create,
    #[serde(bound = "")]
    Read(SerializableId),
    #[serde(bound = "")]
    Edit(SerializableId),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CrudSimpleView {
    List,
    Create,
    Read,
    Edit,
}

impl<ReadId, UpdateId> Into<SerializableCrudView> for CrudView<ReadId, UpdateId>
where
    ReadId: crudkit_id::Id + Serialize + DeserializeOwned,
    UpdateId: crudkit_id::Id + Serialize + DeserializeOwned,
{
    fn into(self) -> SerializableCrudView {
        match self {
            CrudView::List => SerializableCrudView::List,
            CrudView::Create => SerializableCrudView::Create,
            CrudView::Read(id) => SerializableCrudView::Read(id.into_serializable_id()),
            CrudView::Edit(id) => SerializableCrudView::Edit(id.into_serializable_id()),
        }
    }
}

impl<ReadId, UpdateId> Into<CrudSimpleView> for CrudView<ReadId, UpdateId>
where
    ReadId: crudkit_id::Id + Serialize + DeserializeOwned,
    UpdateId: crudkit_id::Id + Serialize + DeserializeOwned,
{
    fn into(self) -> CrudSimpleView {
        match self {
            CrudView::List => CrudSimpleView::List,
            CrudView::Create => CrudSimpleView::Create,
            CrudView::Read(_) => CrudSimpleView::Read,
            CrudView::Edit(_) => CrudSimpleView::Edit,
        }
    }
}

impl<ReadId, UpdateId> Default for CrudView<ReadId, UpdateId>
where
    ReadId: crudkit_id::Id + Serialize + DeserializeOwned,
    UpdateId: crudkit_id::Id + Serialize + DeserializeOwned,
{
    fn default() -> Self {
        Self::List
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FieldMode {
    Display,
    Readable,
    Editable,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeletableModel<
    ReadModel: CrudDataTrait + CrudIdTrait,
    UpdateModel: CrudDataTrait + CrudIdTrait,
> {
    Read(ReadModel),
    Update(UpdateModel),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HeaderOptions {
    pub display_name: Cow<'static, str>,
    pub min_width: bool, // TODO: Rename to fit content.
    pub ordering_allowed: bool,
    pub date_time_display: DateTimeDisplay,
}

// TODO: we might want to use the builder pattern instead of relying on ..Default.default()
impl Default for HeaderOptions {
    fn default() -> Self {
        Self {
            display_name: Cow::Borrowed(""),
            min_width: false,
            ordering_allowed: true,
            date_time_display: DateTimeDisplay::LocalizedLocal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DateTimeDisplay {
    IsoUtc,
    LocalizedLocal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Label {
    pub name: String,
}

impl Label {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldOptions {
    pub disabled: bool,
    pub label: Option<Label>,
    pub date_time_display: DateTimeDisplay,
    //validations: Vec<u32>,
}

impl Default for FieldOptions {
    fn default() -> Self {
        Self {
            disabled: false,
            label: None,
            date_time_display: DateTimeDisplay::LocalizedLocal,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct OrderByUpdateOptions {
    pub append: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Elem<T: CrudDataTrait> {
    // serde bound used as described in: https://github.com/serde-rs/serde/issues/1296
    #[serde(bound = "")]
    Enclosing(Enclosing<T>),
    Field((T::Field, FieldOptions)),
    Separator,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnyElem {
    Enclosing(AnyEnclosing),
    Field((AnyField, FieldOptions)),
    Separator,
}

pub type TabId = Cow<'static, str>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tab<T: CrudDataTrait> {
    /// A unique identifier for this tab.
    pub id: TabId,
    pub label: Label,
    #[serde(bound = "")]
    pub group: Group<T>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnyTab {
    /// A unique identifier for this tab.
    pub id: TabId,
    pub label: Label,
    pub group: AnyGroup,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Enclosing<T: CrudDataTrait> {
    #[serde(bound = "")]
    None(Group<T>),
    #[serde(bound = "")]
    Tabs(Vec<Tab<T>>),
    #[serde(bound = "")]
    Card(Group<T>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnyEnclosing {
    None(AnyGroup),
    Tabs(Vec<AnyTab>),
    Card(AnyGroup),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Layout {
    Columns1,
    Columns2,
    Columns3,
    Columns4,
}

impl Default for Layout {
    fn default() -> Self {
        Self::Columns2
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group<T: CrudDataTrait> {
    pub layout: Layout,
    // serde bound used as described in: https://github.com/serde-rs/serde/issues/1296
    #[serde(bound = "")]
    pub children: Vec<Elem<T>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnyGroup {
    pub layout: Layout,
    pub children: Vec<AnyElem>,
}
