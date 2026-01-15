#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

pub mod crud_action;
pub mod crud_action_buttons;
pub mod crud_action_context;
pub mod crud_create_view;
pub mod crud_delete_many_modal;
pub mod crud_delete_modal;
pub mod crud_edit_view;
pub mod crud_field;
pub mod crud_field_label;
pub mod crud_fields;
pub mod crud_instance;
pub mod crud_instance_config;
pub mod crud_instance_mgr;
pub mod crud_leave_modal;
pub mod crud_list_view;
pub mod crud_pagination;
pub mod crud_read_view;
pub mod crud_table;
pub mod crud_table_body;
pub mod crud_table_footer;
pub mod crud_table_header;
pub mod fields;
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
pub use crudkit_core;
pub use crudkit_id;
pub use crudkit_validation;
pub use crudkit_web;
pub use crudkit_websocket;

use crudkit_core::{FieldValue, TimeDuration, Value};
use leptos::prelude::*;

// TODO: This prelude should only contain types always required when using the lib.
pub mod prelude {
    pub use crudkit_condition;
    pub use crudkit_core;
    pub use crudkit_core::*;
    pub use crudkit_id;
    pub use crudkit_id::*;
    pub use crudkit_validation;
    pub use crudkit_web;
    pub use crudkit_websocket;

    pub use crudkit_web::prelude::*;
    pub use crudkit_web::dynamic::prelude::*;

    pub use derive_crud_action_payload::CkActionPayload;
    pub use derive_crud_resource::CkResource;
    pub use derive_crudkit_id::CkId;
    pub use derive_field::CkField;

    pub use super::IntoReactiveValue;
    pub use super::ReactiveValue;
    pub use super::SignalsTrait;

    pub use super::crud_action::CrudAction;
    pub use super::crud_action::CrudActionAftermath;
    pub use super::crud_action::CrudEntityAction;
    pub use super::crud_action::EntityActionViewInput;
    pub use super::crud_action::ResourceActionViewInput;

    pub use super::crud_instance::CrudInstance;
    pub use super::crud_instance_config::CreateElements;
    pub use super::crud_instance_config::CrudInstanceConfig;
    pub use super::crud_instance_config::CrudParentConfig;
}

/// Anything that can be created from a HashMap of `ReactiveValue`s.
// TODO: Rename to `Signals`.
pub trait SignalsTrait {
    type Field;

    // TODO: Could also specify additional into_ fn taking `self`.
    //fn to_signals(&self) -> std::collections::HashMap<Self::Field, ReactiveValue>;
    fn from_signals(signals: &std::collections::HashMap<Self::Field, ReactiveValue>) -> Self;

    fn from_signals_untracked(
        signals: &std::collections::HashMap<Self::Field, ReactiveValue>,
    ) -> Self;
}

/// Theoretically, all `Value` types are already defined through `crudkit_core::Value`.
/// But we want to have fine-grained reactivity in this library. Therefore, this type exists,
/// mapping each `crudkit_core::Value` to the same type wrapped inside an `RwSignal`. This allows
/// us to reactively work with individual fields of an entity, not just the whole entity itself.
// TODO: Move into own module
#[derive(Debug, Clone, Copy)]
pub enum ReactiveValue {
    Void(RwSignal<()>),

    Bool(RwSignal<bool>),
    OptionalBool(RwSignal<Option<bool>>),

    I8(RwSignal<i8>),
    I16(RwSignal<i16>),
    I32(RwSignal<i32>),
    I64(RwSignal<i64>),
    I128(RwSignal<i128>),
    OptionalI8(RwSignal<Option<i8>>),
    OptionalI16(RwSignal<Option<i16>>),
    OptionalI32(RwSignal<Option<i32>>),
    OptionalI64(RwSignal<Option<i64>>),
    OptionalI128(RwSignal<Option<i128>>),

    U8(RwSignal<u8>),
    U16(RwSignal<u16>),
    U32(RwSignal<u32>),
    U64(RwSignal<u64>),
    U128(RwSignal<u128>),
    OptionalU8(RwSignal<Option<u8>>),
    OptionalU16(RwSignal<Option<u16>>),
    OptionalU32(RwSignal<Option<u32>>),
    OptionalU64(RwSignal<Option<u64>>),
    OptionalU128(RwSignal<Option<u128>>),

    F32(RwSignal<f32>),
    F64(RwSignal<f64>),
    OptionalF32(RwSignal<Option<f32>>),
    OptionalF64(RwSignal<Option<f64>>),

    String(RwSignal<String>),
    OptionalString(RwSignal<Option<String>>),

    // Ecosystem support.
    // -- serde
    Json(RwSignal<serde_json::Value>),
    OptionalJson(RwSignal<Option<serde_json::Value>>),
    // -- uuid
    Uuid(RwSignal<uuid::Uuid>),
    OptionalUuid(RwSignal<Option<uuid::Uuid>>),
    // -- time
    PrimitiveDateTime(RwSignal<time::PrimitiveDateTime>),
    OffsetDateTime(RwSignal<time::OffsetDateTime>),
    Duration(RwSignal<TimeDuration>),
    OptionalPrimitiveDateTime(RwSignal<Option<time::PrimitiveDateTime>>),
    OptionalOffsetDateTime(RwSignal<Option<time::OffsetDateTime>>),
    OptionalDuration(RwSignal<Option<TimeDuration>>),

    // Extension support.
    Other(RwSignal<Box<dyn FieldValue>>),
}

pub trait IntoReactiveValue {
    fn into_reactive_value(self) -> ReactiveValue;
}

impl IntoReactiveValue for Value {
    fn into_reactive_value(self) -> ReactiveValue {
        match self {
            Value::Void(()) => ReactiveValue::Void(RwSignal::new(())),

            Value::Bool(value) => ReactiveValue::Bool(RwSignal::new(value)),
            Value::OptionalBool(value) => ReactiveValue::OptionalBool(RwSignal::new(value)),

            Value::U8(value) => ReactiveValue::U8(RwSignal::new(value)),
            Value::U16(value) => ReactiveValue::U16(RwSignal::new(value)),
            Value::U32(value) => ReactiveValue::U32(RwSignal::new(value)),
            Value::U64(value) => ReactiveValue::U64(RwSignal::new(value)),
            Value::U128(value) => ReactiveValue::U128(RwSignal::new(value)),
            Value::OptionalU8(value) => ReactiveValue::OptionalU8(RwSignal::new(value)),
            Value::OptionalU16(value) => ReactiveValue::OptionalU16(RwSignal::new(value)),
            Value::OptionalU32(value) => ReactiveValue::OptionalU32(RwSignal::new(value)),
            Value::OptionalU64(value) => ReactiveValue::OptionalU64(RwSignal::new(value)),
            Value::OptionalU128(value) => ReactiveValue::OptionalU128(RwSignal::new(value)),

            Value::I8(value) => ReactiveValue::I8(RwSignal::new(value)),
            Value::I16(value) => ReactiveValue::I16(RwSignal::new(value)),
            Value::I32(value) => ReactiveValue::I32(RwSignal::new(value)),
            Value::I64(value) => ReactiveValue::I64(RwSignal::new(value)),
            Value::I128(value) => ReactiveValue::I128(RwSignal::new(value)),
            Value::OptionalI8(value) => ReactiveValue::OptionalI8(RwSignal::new(value)),
            Value::OptionalI16(value) => ReactiveValue::OptionalI16(RwSignal::new(value)),
            Value::OptionalI32(value) => ReactiveValue::OptionalI32(RwSignal::new(value)),
            Value::OptionalI64(value) => ReactiveValue::OptionalI64(RwSignal::new(value)),
            Value::OptionalI128(value) => ReactiveValue::OptionalI128(RwSignal::new(value)),

            Value::U8Vec(value) => unimplemented!("support vec"),
            Value::I32Vec(value) => unimplemented!("support vec"),
            Value::I64Vec(value) => unimplemented!("support vec"),

            Value::F32(value) => ReactiveValue::F32(RwSignal::new(value)),
            Value::F64(value) => ReactiveValue::F64(RwSignal::new(value)),
            Value::OptionalF32(value) => ReactiveValue::OptionalF32(RwSignal::new(value)),
            Value::OptionalF64(value) => ReactiveValue::OptionalF64(RwSignal::new(value)),

            Value::String(value) => ReactiveValue::String(RwSignal::new(value)),
            Value::OptionalString(value) => ReactiveValue::OptionalString(RwSignal::new(value)),

            Value::Json(value) => ReactiveValue::Json(RwSignal::new(value)),
            Value::OptionalJson(value) => ReactiveValue::OptionalJson(RwSignal::new(value)),

            Value::Uuid(value) => ReactiveValue::Uuid(RwSignal::new(value)),
            Value::OptionalUuid(value) => ReactiveValue::OptionalUuid(RwSignal::new(value)),

            Value::PrimitiveDateTime(value) => {
                ReactiveValue::PrimitiveDateTime(RwSignal::new(value))
            }
            Value::OffsetDateTime(value) => ReactiveValue::OffsetDateTime(RwSignal::new(value)),
            Value::Duration(value) => ReactiveValue::Duration(RwSignal::new(value)),
            Value::OptionalPrimitiveDateTime(value) => {
                ReactiveValue::OptionalPrimitiveDateTime(RwSignal::new(value))
            }
            Value::OptionalOffsetDateTime(value) => {
                ReactiveValue::OptionalOffsetDateTime(RwSignal::new(value))
            }
            Value::OptionalDuration(value) => ReactiveValue::OptionalDuration(RwSignal::new(value)),

            Value::Other(value) => ReactiveValue::Other(RwSignal::new(value)),
        }
    }
}

impl ReactiveValue {
    /// Allows to dynamically set a new value based on any given `crudkit_core::Value`.
    /// Make sure that only appropriate values are passed, as this function
    /// may *panic* if `Value` is not of the same* or otherwise compliant variant as this ReactiveValue.
    /// Uses the `crudkit_core::Value::take_*` functions to get the expected type out of `v` or fail.
    pub fn set(&self, v: Value) {
        match self {
            ReactiveValue::Void(_) => panic!("Calling `set` on a Void value is not allowed."),

            ReactiveValue::Bool(sig) => sig.set(v.take_bool()),
            ReactiveValue::OptionalBool(sig) => sig.set(v.take_optional_bool()),

            ReactiveValue::U8(sig) => sig.set(v.take_u8()),
            ReactiveValue::U16(sig) => sig.set(v.take_u16()),
            ReactiveValue::U32(sig) => sig.set(v.take_u32()),
            ReactiveValue::U64(sig) => sig.set(v.take_u64()),
            ReactiveValue::U128(sig) => sig.set(v.take_u128()),
            ReactiveValue::OptionalU8(sig) => sig.set(v.take_optional_u8()),
            ReactiveValue::OptionalU16(sig) => sig.set(v.take_optional_u16()),
            ReactiveValue::OptionalU32(sig) => sig.set(v.take_optional_u32()),
            ReactiveValue::OptionalU64(sig) => sig.set(v.take_optional_u64()),
            ReactiveValue::OptionalU128(sig) => sig.set(v.take_optional_u128()),

            ReactiveValue::I8(sig) => sig.set(v.take_i8()),
            ReactiveValue::I16(sig) => sig.set(v.take_i16()),
            ReactiveValue::I32(sig) => sig.set(v.take_i32()),
            ReactiveValue::I64(sig) => sig.set(v.take_i64()),
            ReactiveValue::I128(sig) => sig.set(v.take_i128()),
            ReactiveValue::OptionalI8(sig) => sig.set(v.take_optional_i8()),
            ReactiveValue::OptionalI16(sig) => sig.set(v.take_optional_i16()),
            ReactiveValue::OptionalI32(sig) => sig.set(v.take_optional_i32()),
            ReactiveValue::OptionalI64(sig) => sig.set(v.take_optional_i64()),
            ReactiveValue::OptionalI128(sig) => sig.set(v.take_optional_i128()),

            ReactiveValue::F32(sig) => sig.set(v.take_f32()),
            ReactiveValue::F64(sig) => sig.set(v.take_f64()),
            ReactiveValue::OptionalF32(sig) => sig.set(v.take_optional_f32()),
            ReactiveValue::OptionalF64(sig) => sig.set(v.take_optional_f64()),

            ReactiveValue::String(sig) => sig.set(v.take_string()),
            ReactiveValue::OptionalString(sig) => sig.set(v.take_optional_string()),

            ReactiveValue::Json(sig) => sig.set(v.take_json_value()),
            ReactiveValue::OptionalJson(sig) => sig.set(v.take_optional_json_value()),

            ReactiveValue::Uuid(sig) => sig.set(v.take_uuid()),
            ReactiveValue::OptionalUuid(sig) => sig.set(v.take_optional_uuid()),

            ReactiveValue::PrimitiveDateTime(sig) => sig.set(v.take_primitive_date_time()),
            ReactiveValue::OffsetDateTime(sig) => sig.set(v.take_offset_date_time()),
            ReactiveValue::Duration(sig) => sig.set(v.take_duration()),
            ReactiveValue::OptionalPrimitiveDateTime(sig) => {
                sig.set(v.take_optional_primitive_date_time())
            }
            ReactiveValue::OptionalOffsetDateTime(sig) => {
                sig.set(v.take_optional_offset_date_time())
            }
            ReactiveValue::OptionalDuration(sig) => sig.set(v.take_optional_duration()),

            ReactiveValue::Other(sig) => sig.set(v.take_other()),
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

    pub fn expect_custom(self) -> RwSignal<Box<dyn FieldValue>> {
        match self {
            ReactiveValue::Other(sig) => sig,
            _ => panic!("Expected ReactiveValue of variant: Custom"),
        }
    }
}
