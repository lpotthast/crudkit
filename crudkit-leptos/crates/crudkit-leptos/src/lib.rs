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
pub mod crud_progress_bar;
//pub mod crud_quicksearch;
pub mod crud_read_view;
//pub mod crud_related_field;
//pub mod crud_relation;
//pub mod crud_reset_field;
pub mod crud_select;
//pub mod crud_select_field;
//pub mod crud_slider;
pub mod crud_table;
pub mod crud_table_body;
pub mod crud_table_footer;
pub mod crud_table_header;
//pub mod crud_tiptap_editor;

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
pub use crudkit_websocket;

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
    pub use super::crud_progress_bar::CrudProgressBar;
    pub use super::crud_read_view::CrudReadView;
    pub use super::crud_select::Selection;
    pub use super::crud_table::CrudTable;
    pub use super::crud_table_body::CrudTableBody;
    pub use super::crud_table_footer::CrudTableFooter;
    pub use super::crud_table_header::CrudTableHeader;
}
