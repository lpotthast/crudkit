#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

pub mod dynamic;
pub mod generic;
pub mod shared;
pub mod stores;
/*
* Reexport common modules.
* This allows the user to only
*
* - `use crudkit_yew::prelude::*` and
* - derive all common proc macros
*
* without the need to add more use declaration or
* to manually depend on other crud crates such as "crudkit_id",
* which are required for many derive macro implementations.
*/
pub use crudkit_condition;
pub use crudkit_id;
pub use crudkit_shared;
use crudkit_shared::TimeDuration;
pub use crudkit_validation;
pub use crudkit_web;
use crudkit_web::value::{JsonValue, Value};
use crudkit_web::CrudSelectableTrait;
pub use crudkit_websocket;
use leptos::prelude::*;

// TODO: This prelude should only contain types always required when using the lib.
pub mod prelude {
    pub use crudkit_condition;
    pub use crudkit_id;
    pub use crudkit_id::*;
    pub use crudkit_shared;
    pub use crudkit_shared::*;
    pub use crudkit_validation;
    pub use crudkit_web;
    pub use crudkit_websocket;

    pub use derive_crud_action_payload::CkActionPayload;
    pub use derive_crud_resource::CkResource;
    pub use derive_crud_selectable::CkSelectable;
    pub use derive_crudkit_id::CkId;
    pub use derive_field::CkField;
    pub use derive_field_signals::CkFieldSignals;
    pub use derive_field_value::CkFieldValue;

    pub use super::IntoReactiveValue;
    pub use super::ReactiveValue;
    pub use super::SignalsTrait;
}

/// Anything that can be created from a HashMap of `ReactiveValue`s.
pub trait SignalsTrait {
    type Field;

    // TODO: Could also specify additional into_ fn taking `self`.
    //fn to_signals(&self) -> std::collections::HashMap<Self::Field, ReactiveValue>;
    fn from_signals(signals: &std::collections::HashMap<Self::Field, ReactiveValue>) -> Self;

    fn from_signals_untracked(
        signals: &std::collections::HashMap<Self::Field, ReactiveValue>,
    ) -> Self;
}

/// Theoretically, all `Value` types are already defined through `crudkit_web::Value`.
/// But we want to have fine-grained reactivity in this library. Therefore, this type exists,
/// mapping each `crudkit_web::Value` to the same type wrapped inside an `RwSignal`. This allows
/// us to reactively work with individual fields of an entity, not just the whole entity itself.
// TODO: Move into own module
#[derive(Debug, Clone, Copy)]
pub enum ReactiveValue {
    String(RwSignal<String>),
    OptionalString(RwSignal<Option<String>>),
    Text(RwSignal<String>),    // TODO: Add optional text!
    Json(RwSignal<JsonValue>), // TODO: Add optional json value
    OptionalJson(RwSignal<Option<JsonValue>>),
    Uuid(RwSignal<uuid::Uuid>), // TODO: Add OptionalUuid variant
    I32(RwSignal<i32>),
    U32(RwSignal<u32>),
    I64(RwSignal<i64>),
    U64(RwSignal<u64>),
    I128(RwSignal<i128>),
    U128(RwSignal<u128>),
    OptionalI32(RwSignal<Option<i32>>),
    OptionalI64(RwSignal<Option<i64>>),
    OptionalU32(RwSignal<Option<u32>>),
    OptionalU64(RwSignal<Option<u64>>),
    F32(RwSignal<f32>),
    F64(RwSignal<f64>),
    Bool(RwSignal<bool>),
    // Specialized bool-case, render as a green check mark if false and an orange exclamation mark if true.
    ValidationStatus(RwSignal<bool>),
    PrimitiveDateTime(RwSignal<time::PrimitiveDateTime>),
    OffsetDateTime(RwSignal<time::OffsetDateTime>),
    Duration(RwSignal<TimeDuration>),
    OptionalPrimitiveDateTime(RwSignal<Option<time::PrimitiveDateTime>>),
    OptionalOffsetDateTime(RwSignal<Option<time::OffsetDateTime>>),
    OptionalDuration(RwSignal<Option<TimeDuration>>),
    OneToOneRelation(RwSignal<Option<u32>>),
    Reference(RwSignal<Vec<Box<dyn crudkit_id::IdField>>>),
    Custom(RwSignal<()>),
    Select(RwSignal<Box<dyn CrudSelectableTrait>>),
    Multiselect(RwSignal<Vec<Box<dyn CrudSelectableTrait>>>),
    OptionalSelect(RwSignal<Option<Box<dyn CrudSelectableTrait>>>),
    OptionalMultiselect(RwSignal<Option<Vec<Box<dyn CrudSelectableTrait>>>>),
    //Select(Box<dyn CrudSelectableSource<Selectable = dyn CrudSelectableTrait>>),
}

pub trait IntoReactiveValue {
    fn into_reactive_value(self) -> ReactiveValue;
}

impl IntoReactiveValue for Value {
    fn into_reactive_value(self) -> ReactiveValue {
        match self {
            Value::String(value) => ReactiveValue::String(RwSignal::new(value)),
            Value::OptionalString(value) => ReactiveValue::OptionalString(RwSignal::new(value)),
            Value::Text(value) => ReactiveValue::Text(RwSignal::new(value)),
            Value::Json(value) => ReactiveValue::Json(RwSignal::new(value)),
            Value::OptionalJson(value) => ReactiveValue::OptionalJson(RwSignal::new(value)),
            Value::Uuid(value) => ReactiveValue::Uuid(RwSignal::new(value)),
            Value::I32(value) => ReactiveValue::I32(RwSignal::new(value)),
            Value::U32(value) => ReactiveValue::U32(RwSignal::new(value)),
            Value::I64(value) => ReactiveValue::I64(RwSignal::new(value)),
            Value::U64(value) => ReactiveValue::U64(RwSignal::new(value)),
            Value::I128(value) => ReactiveValue::I128(RwSignal::new(value)),
            Value::U128(value) => ReactiveValue::U128(RwSignal::new(value)),
            Value::OptionalI32(value) => ReactiveValue::OptionalI32(RwSignal::new(value)),
            Value::OptionalI64(value) => ReactiveValue::OptionalI64(RwSignal::new(value)),
            Value::OptionalU32(value) => ReactiveValue::OptionalU32(RwSignal::new(value)),
            Value::OptionalU64(value) => ReactiveValue::OptionalU64(RwSignal::new(value)),
            Value::F32(value) => ReactiveValue::F32(RwSignal::new(value)),
            Value::F64(value) => ReactiveValue::F64(RwSignal::new(value)),
            Value::Bool(value) => ReactiveValue::Bool(RwSignal::new(value)),
            Value::ValidationStatus(value) => ReactiveValue::ValidationStatus(RwSignal::new(value)),
            Value::PrimitiveDateTime(value) => {
                ReactiveValue::PrimitiveDateTime(RwSignal::new(value))
            }
            Value::OffsetDateTime(value) => ReactiveValue::OffsetDateTime(RwSignal::new(value)),
            Value::OptionalPrimitiveDateTime(value) => {
                ReactiveValue::OptionalPrimitiveDateTime(RwSignal::new(value))
            }
            Value::OptionalOffsetDateTime(value) => {
                ReactiveValue::OptionalOffsetDateTime(RwSignal::new(value))
            }
            Value::OneToOneRelation(value) => ReactiveValue::OneToOneRelation(RwSignal::new(value)),
            Value::Reference(value) => ReactiveValue::Reference(RwSignal::new(value)),
            Value::Custom(value) => ReactiveValue::Custom(RwSignal::new(value)),
            Value::Select(value) => ReactiveValue::Select(RwSignal::new(value)),
            Value::Multiselect(value) => ReactiveValue::Multiselect(RwSignal::new(value)),
            Value::OptionalSelect(value) => ReactiveValue::OptionalSelect(RwSignal::new(value)),
            Value::OptionalMultiselect(value) => {
                ReactiveValue::OptionalMultiselect(RwSignal::new(value))
            }
            Value::Duration(value) => ReactiveValue::Duration(RwSignal::new(value)),
            Value::OptionalDuration(value) => ReactiveValue::OptionalDuration(RwSignal::new(value)),
        }
    }
}

impl ReactiveValue {
    /// Allows to dynamically set a new value based on any given `crudkit_web::Value`.
    /// Make sure that only appropriate values are passed, as this function
    /// may *panic* if `Value` is not of the same* or otherwise compliant variant as this ReactiveValue.
    /// Uses the `crudkit_web::Value::take_*` functions to get the expected type out of `v` or fail.
    pub fn set(&self, v: Value) {
        match self {
            ReactiveValue::String(sig) => sig.set(v.take_string()),
            ReactiveValue::OptionalString(sig) => sig.set(v.take_optional_string()),
            ReactiveValue::Text(sig) => sig.set(v.take_text()),
            ReactiveValue::Json(sig) => sig.set(v.take_json_value()),
            ReactiveValue::OptionalJson(sig) => sig.set(v.take_optional_json_value()),
            ReactiveValue::Uuid(sig) => sig.set(v.take_uuid()),
            ReactiveValue::I32(sig) => sig.set(v.take_i32()),
            ReactiveValue::U32(sig) => sig.set(v.take_u32()),
            ReactiveValue::I64(sig) => sig.set(v.take_i64()),
            ReactiveValue::U64(sig) => sig.set(v.take_u64()),
            ReactiveValue::I128(sig) => sig.set(v.take_i128()),
            ReactiveValue::U128(sig) => sig.set(v.take_u128()),
            ReactiveValue::OptionalU32(sig) => sig.set(v.take_optional_u32()),
            ReactiveValue::OptionalI32(sig) => sig.set(v.take_optional_i32()),
            ReactiveValue::OptionalI64(sig) => sig.set(v.take_optional_i64()),
            ReactiveValue::OptionalU64(sig) => sig.set(v.take_optional_u64()),
            ReactiveValue::F32(sig) => sig.set(v.take_f32()),
            ReactiveValue::F64(sig) => sig.set(v.take_f64()),
            ReactiveValue::Bool(sig) => sig.set(v.take_bool()),
            ReactiveValue::ValidationStatus(sig) => sig.set(v.take_validation_status()),
            ReactiveValue::PrimitiveDateTime(sig) => sig.set(v.take_primitive_date_time()),
            ReactiveValue::OffsetDateTime(sig) => sig.set(v.take_offset_date_time()),
            ReactiveValue::OptionalPrimitiveDateTime(sig) => {
                sig.set(v.take_optional_primitive_date_time())
            }
            ReactiveValue::OptionalOffsetDateTime(sig) => {
                sig.set(v.take_optional_offset_date_time())
            }
            ReactiveValue::OneToOneRelation(sig) => sig.set(v.take_one_to_one_relation()),
            ReactiveValue::Reference(sig) => sig.set(v.take_reference()),
            ReactiveValue::Custom(sig) => sig.set(v.take_custom()),
            ReactiveValue::Select(sig) => sig.set(v.take_select()),
            ReactiveValue::Multiselect(sig) => sig.set(v.take_multiselect()),
            ReactiveValue::OptionalSelect(sig) => sig.set(v.take_optional_select_downcast_to()),
            ReactiveValue::OptionalMultiselect(sig) => {
                sig.set(v.take_optional_multiselect_downcast_to())
            }
            ReactiveValue::Duration(sig) => sig.set(v.take_duration()),
            ReactiveValue::OptionalDuration(sig) => sig.set(v.take_optional_duration()),
        }
    }

    pub fn expect_string(self) -> RwSignal<String> {
        match self {
            ReactiveValue::String(sig) => sig,
            _ => panic!("Expected ReactiveValue of variant: String"),
        }
    }

    pub fn expect_optional_string(self) -> RwSignal<Option<String>> {
        match self {
            ReactiveValue::OptionalString(sig) => sig,
            _ => panic!("Expected ReactiveValue of variant: OptionalString"),
        }
    }

    pub fn expect_bool(self) -> RwSignal<bool> {
        match self {
            ReactiveValue::Bool(sig) => sig,
            _ => panic!("Expected ReactiveValue of variant: Bool"),
        }
    }

    pub fn expect_i32(self) -> RwSignal<i32> {
        match self {
            ReactiveValue::I32(sig) => sig,
            _ => panic!("Expected ReactiveValue of variant: I32"),
        }
    }

    pub fn expect_i64(self) -> RwSignal<i64> {
        match self {
            ReactiveValue::I64(sig) => sig,
            _ => panic!("Expected ReactiveValue of variant: I64"),
        }
    }

    pub fn expect_optional_i64(self) -> RwSignal<Option<i64>> {
        match self {
            ReactiveValue::OptionalI64(sig) => sig,
            _ => panic!("Expected ReactiveValue of variant: OptionalI64"),
        }
    }

    pub fn expect_select(self) -> RwSignal<Box<dyn CrudSelectableTrait>> {
        match self {
            ReactiveValue::Select(sig) => sig,
            _ => panic!("Expected ReactiveValue of variant: Select"),
        }
    }

    pub fn expect_custom(self) -> RwSignal<()> {
        match self {
            ReactiveValue::Custom(sig) => sig,
            _ => panic!("Expected ReactiveValue of variant: Custom"),
        }
    }
}
