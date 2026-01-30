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
pub use crudkit_core;
pub use crudkit_web;

use crudkit_core::Value;
use leptos::prelude::*;

pub mod prelude {
    pub use crudkit_core;
    pub use crudkit_core::collaboration;
    pub use crudkit_core::condition;
    pub use crudkit_core::id;
    pub use crudkit_core::id::*;
    pub use crudkit_core::validation;
    pub use crudkit_core::*;
    pub use crudkit_web;
    pub use crudkit_web::prelude::*;

    // Explicitly re-export Model from crudkit_web to resolve ambiguity
    // (both crudkit_core and crudkit_web export Model).
    pub use crudkit_web::Model;

    pub use crudkit_core_macros::CkId;
    pub use crudkit_web_macros::{CkActionPayload, CkField, CkResource};

    pub use super::crud_action::{
        CrudAction, CrudActionAftermath, CrudEntityAction, EntityActionViewInput,
        ResourceActionViewInput,
    };
    pub use super::crud_instance::CrudInstance;
    pub use super::crud_instance_config::{CreateElements, CrudInstanceConfig, CrudParentConfig};
    pub use super::ReactiveField;
}

/// A reactive field holding a `Value` signal.
///
/// This provides fine-grained reactivity for individual fields. The inner signal
/// holds `Value` directly, where `Value::Null` represents an absent/null value for optional fields.
#[derive(Debug, Clone, Copy)]
pub struct ReactiveField {
    /// The reactive signal holding the field value.
    pub value: RwSignal<Value>,
}

impl ReactiveField {
    /// Creates a new `ReactiveField` with the given initial value.
    pub fn new(value: Value) -> Self {
        Self {
            value: RwSignal::new(value),
        }
    }

    /// Sets the value.
    pub fn set(&self, v: Value) {
        self.value.set(v);
    }

    /// Gets the current value.
    pub fn get(&self) -> Value {
        self.value.get()
    }

    /// Gets the current value without tracking.
    pub fn get_untracked(&self) -> Value {
        self.value.get_untracked()
    }
}

impl From<Value> for ReactiveField {
    fn from(v: Value) -> Self {
        ReactiveField::new(v)
    }
}
