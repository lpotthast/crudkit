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
pub mod custom_field;
//pub mod crud_nested_instance;
pub mod crud_read_view;
//pub mod crud_related_field;
pub mod crud_table;
pub mod crud_table_body;
pub mod crud_table_footer;
pub mod crud_table_header;

pub mod prelude {
    pub use crate::prelude::*;

    pub use crudkit_web::generic::prelude::*;
}
