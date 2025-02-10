pub mod crud_action;
pub mod crud_action_buttons;
pub mod crud_action_context;
pub mod crud_create_view;
pub mod crud_delete_modal;
pub mod crud_edit_view;
pub mod crud_field;
pub mod crud_fields;
pub mod crud_instance;
pub mod crud_instance_config;
pub mod crud_list_view;
pub mod crud_read_view;
pub mod crud_table;
pub mod crud_table_body;
pub mod crud_table_footer;
pub mod crud_table_header;
pub mod custom_field;

pub mod prelude {
    pub use crate::prelude::*;

    pub use crudkit_web::dynamic::prelude::*;

    pub use super::crud_action::CrudAction;
    pub use super::crud_action::CrudActionAftermath;
    pub use super::crud_action::CrudEntityAction;
    pub use super::crud_action::EntityModalGeneration;
    pub use super::crud_action::ModalGeneration;

    pub use super::crud_instance::CrudInstance;
    pub use super::crud_instance_config::CreateElements;
    pub use super::crud_instance_config::CrudInstanceConfig;
    pub use super::crud_instance_config::CrudStaticInstanceConfig;
}
