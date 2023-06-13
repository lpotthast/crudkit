#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

pub mod crud_action;
pub mod crud_action_context;
pub mod crud_create_view;
pub mod crud_delete_modal;
pub mod crud_edit_view;
pub mod crud_field_label;
pub mod crud_field_leptos;
pub mod crud_fields;
pub mod crud_instance;
pub mod crud_instance_config;
pub mod crud_leave_modal;
pub mod crud_list_view;
//pub mod crud_nested_instance;
pub mod crud_pagination;
pub mod crud_read_view;
//pub mod crud_related_field;
//pub mod crud_slider;
pub mod crud_table;
pub mod crud_table_body;
pub mod crud_table_footer;
pub mod crud_table_header;

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
pub use crudkit_validation;
pub use crudkit_web;
pub use crudkit_web::prelude::*;
use crudkit_web::JsonValue;
pub use crudkit_websocket;
use leptos::*;

pub mod prelude {
    pub use crudkit_condition;
    pub use crudkit_id;
    pub use crudkit_shared;
    pub use crudkit_validation;
    pub use crudkit_web; // TODO: Should this be removed?
    pub use crudkit_web::prelude::*; // TODO: Should this be removed?
    pub use crudkit_websocket;

    pub use derive_crud_action_payload::CkActionPayload;
    pub use derive_crud_resource::CkResource;
    pub use derive_crud_selectable::CkSelectable;
    pub use derive_crudkit_id::CkId;
    pub use derive_field::CkField;
    pub use derive_field_value::CkFieldValue;

    pub use super::crud_action::CrudAction;
    pub use super::crud_action::CrudActionAftermath;
    pub use super::crud_action::CrudActionTrait;
    pub use super::crud_action::CrudEntityAction;
    pub use super::crud_action::States;
    pub use super::crud_action_context::CrudActionContext;
    pub use super::crud_create_view::CrudCreateView;
    pub use super::crud_edit_view::CrudEditView;
    pub use super::crud_field_label::CrudFieldLabel;
    pub use super::crud_field_leptos::CrudField;
    pub use super::crud_fields::CrudFields;
    pub use super::crud_instance::CrudInstance;
    pub use super::crud_instance_config::CreateElements;
    pub use super::crud_instance_config::CrudInstanceConfig;
    pub use super::crud_instance_config::CrudStaticInstanceConfig;
    pub use super::crud_leave_modal::CrudLeaveModal;
    pub use super::crud_list_view::CrudListView;
    pub use super::crud_pagination::CrudPagination;
    pub use super::crud_read_view::CrudReadView;
    pub use super::crud_table::CrudTable;
    pub use super::crud_table_body::CrudTableBody;
    pub use super::crud_table_footer::CrudTableFooter;
    pub use super::crud_table_header::CrudTableHeader;
}

// TODO: Move into own module
#[derive(Debug, Clone, Copy)]
pub enum ReactiveValue {
    String(RwSignal<String>),  // TODO: Add optional string!
    Text(RwSignal<String>),    // TODO: Add optional text!
    Json(RwSignal<JsonValue>), // TODO: Add optional json value
    OptionalJson(RwSignal<Option<JsonValue>>),
    UuidV4(RwSignal<uuid::Uuid>), // TODO: Add optional UuidV4 value
    UuidV7(RwSignal<uuid::Uuid>), // TODO: Add optional UuidV7 value
    U32(RwSignal<u32>),
    OptionalU32(RwSignal<Option<u32>>),
    I32(RwSignal<i32>),
    OptionalI32(RwSignal<Option<i32>>),
    I64(RwSignal<i64>),
    OptionalI64(RwSignal<Option<i64>>),
    F32(RwSignal<f32>),
    Bool(RwSignal<bool>),
    // Specialized bool-case, render as a green check mark if false and an orange exclamation mark if true.
    ValidationStatus(RwSignal<bool>),
    PrimitiveDateTime(RwSignal<time::PrimitiveDateTime>),
    OffsetDateTime(RwSignal<time::OffsetDateTime>),
    OptionalPrimitiveDateTime(RwSignal<Option<time::PrimitiveDateTime>>),
    OptionalOffsetDateTime(RwSignal<Option<time::OffsetDateTime>>),
    OneToOneRelation(RwSignal<Option<u32>>),
    NestedTable(RwSignal<Vec<Box<dyn crudkit_id::IdField>>>),
    Custom(RwSignal<()>),
    Select(RwSignal<Box<dyn CrudSelectableTrait>>),
    Multiselect(RwSignal<Vec<Box<dyn CrudSelectableTrait>>>),
    OptionalSelect(RwSignal<Option<Box<dyn CrudSelectableTrait>>>),
    OptionalMultiselect(RwSignal<Option<Vec<Box<dyn CrudSelectableTrait>>>>),
    //Select(Box<dyn CrudSelectableSource<Selectable = dyn CrudSelectableTrait>>),
}

pub trait IntoReactiveValue {
    fn into_reactive_value(self, cx: Scope) -> ReactiveValue;
}

impl IntoReactiveValue for Value {
    fn into_reactive_value(self, cx: Scope) -> ReactiveValue {
        match self {
            Value::String(value) => ReactiveValue::String(create_rw_signal(cx, value)),
            Value::Text(value) => ReactiveValue::Text(create_rw_signal(cx, value)),
            Value::Json(value) => ReactiveValue::Json(create_rw_signal(cx, value)),
            Value::OptionalJson(value) => ReactiveValue::OptionalJson(create_rw_signal(cx, value)),
            Value::UuidV4(value) => ReactiveValue::UuidV4(create_rw_signal(cx, value)),
            Value::UuidV7(value) => ReactiveValue::UuidV7(create_rw_signal(cx, value)),
            Value::U32(value) => ReactiveValue::U32(create_rw_signal(cx, value)),
            Value::OptionalU32(value) => ReactiveValue::OptionalU32(create_rw_signal(cx, value)),
            Value::I32(value) => ReactiveValue::I32(create_rw_signal(cx, value)),
            Value::OptionalI32(value) => ReactiveValue::OptionalI32(create_rw_signal(cx, value)),
            Value::I64(value) => ReactiveValue::I64(create_rw_signal(cx, value)),
            Value::OptionalI64(value) => ReactiveValue::OptionalI64(create_rw_signal(cx, value)),
            Value::F32(value) => ReactiveValue::F32(create_rw_signal(cx, value)),
            Value::Bool(value) => ReactiveValue::Bool(create_rw_signal(cx, value)),
            Value::ValidationStatus(value) => {
                ReactiveValue::ValidationStatus(create_rw_signal(cx, value))
            }
            Value::PrimitiveDateTime(value) => {
                ReactiveValue::PrimitiveDateTime(create_rw_signal(cx, value))
            }
            Value::OffsetDateTime(value) => {
                ReactiveValue::OffsetDateTime(create_rw_signal(cx, value))
            }
            Value::OptionalPrimitiveDateTime(value) => {
                ReactiveValue::OptionalPrimitiveDateTime(create_rw_signal(cx, value))
            }
            Value::OptionalOffsetDateTime(value) => {
                ReactiveValue::OptionalOffsetDateTime(create_rw_signal(cx, value))
            }
            Value::OneToOneRelation(value) => {
                ReactiveValue::OneToOneRelation(create_rw_signal(cx, value))
            }
            Value::NestedTable(value) => ReactiveValue::NestedTable(create_rw_signal(cx, value)),
            Value::Custom(value) => ReactiveValue::Custom(create_rw_signal(cx, value)),
            Value::Select(value) => ReactiveValue::Select(create_rw_signal(cx, value)),
            Value::Multiselect(value) => ReactiveValue::Multiselect(create_rw_signal(cx, value)),
            Value::OptionalSelect(value) => {
                ReactiveValue::OptionalSelect(create_rw_signal(cx, value))
            }
            Value::OptionalMultiselect(value) => {
                ReactiveValue::OptionalMultiselect(create_rw_signal(cx, value))
            }
        }
    }
}
