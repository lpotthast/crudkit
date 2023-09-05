#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

pub mod crud_action;
pub mod crud_action_buttons;
pub mod crud_action_context;
pub mod crud_create_view;
pub mod crud_delete_modal;
pub mod crud_edit_view;
pub mod crud_field_label;
pub mod crud_field;
pub mod crud_fields;
pub mod crud_instance;
pub mod crud_instance_config;
pub mod crud_instance_mgr;
pub mod crud_leave_modal;
pub mod crud_list_view;
//pub mod crud_nested_instance;
pub mod crud_pagination;
pub mod crud_read_view;
//pub mod crud_related_field;
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
    pub use super::crud_field_label::CrudFieldLabelOpt;
    pub use super::crud_field::CrudField;
    pub use super::crud_fields::CrudFields;
    pub use super::crud_instance::CrudInstance;
    pub use super::crud_instance_config::CreateElements;
    pub use super::crud_instance_config::CrudInstanceConfig;
    pub use super::crud_instance_config::CrudStaticInstanceConfig;
    pub use super::crud_instance_mgr::CrudInstanceMgr;
    pub use super::crud_instance_mgr::CrudInstanceMgrContext;
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
            Value::String(value) => ReactiveValue::String(create_rw_signal(value)),
            Value::Text(value) => ReactiveValue::Text(create_rw_signal(value)),
            Value::Json(value) => ReactiveValue::Json(create_rw_signal(value)),
            Value::OptionalJson(value) => ReactiveValue::OptionalJson(create_rw_signal(value)),
            Value::UuidV4(value) => ReactiveValue::UuidV4(create_rw_signal(value)),
            Value::UuidV7(value) => ReactiveValue::UuidV7(create_rw_signal(value)),
            Value::U32(value) => ReactiveValue::U32(create_rw_signal(value)),
            Value::OptionalU32(value) => ReactiveValue::OptionalU32(create_rw_signal(value)),
            Value::I32(value) => ReactiveValue::I32(create_rw_signal(value)),
            Value::OptionalI32(value) => ReactiveValue::OptionalI32(create_rw_signal(value)),
            Value::I64(value) => ReactiveValue::I64(create_rw_signal(value)),
            Value::OptionalI64(value) => ReactiveValue::OptionalI64(create_rw_signal(value)),
            Value::F32(value) => ReactiveValue::F32(create_rw_signal(value)),
            Value::Bool(value) => ReactiveValue::Bool(create_rw_signal(value)),
            Value::ValidationStatus(value) => {
                ReactiveValue::ValidationStatus(create_rw_signal(value))
            }
            Value::PrimitiveDateTime(value) => {
                ReactiveValue::PrimitiveDateTime(create_rw_signal(value))
            }
            Value::OffsetDateTime(value) => ReactiveValue::OffsetDateTime(create_rw_signal(value)),
            Value::OptionalPrimitiveDateTime(value) => {
                ReactiveValue::OptionalPrimitiveDateTime(create_rw_signal(value))
            }
            Value::OptionalOffsetDateTime(value) => {
                ReactiveValue::OptionalOffsetDateTime(create_rw_signal(value))
            }
            Value::OneToOneRelation(value) => {
                ReactiveValue::OneToOneRelation(create_rw_signal(value))
            }
            Value::Reference(value) => ReactiveValue::Reference(create_rw_signal(value)),
            Value::Custom(value) => ReactiveValue::Custom(create_rw_signal(value)),
            Value::Select(value) => ReactiveValue::Select(create_rw_signal(value)),
            Value::Multiselect(value) => ReactiveValue::Multiselect(create_rw_signal(value)),
            Value::OptionalSelect(value) => ReactiveValue::OptionalSelect(create_rw_signal(value)),
            Value::OptionalMultiselect(value) => {
                ReactiveValue::OptionalMultiselect(create_rw_signal(value))
            }
        }
    }
}

impl ReactiveValue {
    pub fn set(&self, v: Value) {
        match self {
            ReactiveValue::String(sig) => sig.set(v.take_string()),
            ReactiveValue::Text(sig) => sig.set(v.take_text()),
            ReactiveValue::Json(sig) => sig.set(v.take_json_value()),
            ReactiveValue::OptionalJson(sig) => sig.set(v.take_optional_json_value()),
            ReactiveValue::UuidV4(sig) => sig.set(v.take_uuid_v4()),
            ReactiveValue::UuidV7(sig) => sig.set(v.take_uuid_v7()),
            ReactiveValue::U32(sig) => sig.set(v.take_u32()),
            ReactiveValue::OptionalU32(sig) => sig.set(v.take_optional_u32()),
            ReactiveValue::I32(sig) => sig.set(v.take_i32()),
            ReactiveValue::OptionalI32(sig) => sig.set(v.take_optional_i32()),
            ReactiveValue::I64(sig) => sig.set(v.take_i64()),
            ReactiveValue::OptionalI64(sig) => sig.set(v.take_optional_i64()),
            ReactiveValue::F32(sig) => sig.set(v.take_f32()),
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
        }
    }
}
