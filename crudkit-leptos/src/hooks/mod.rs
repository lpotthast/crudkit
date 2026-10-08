//! CrudKit's behavior as reactive state.
//!
//! State hooks (`use_crud_*`) return `Copy` handles to reactive state owned by the surrounding
//! [`CrudInstance`](crate::instance::CrudInstance) or by the calling view. They render nothing.
//! The atoms are built on them, and applications can build markup the atoms do not cover on
//! them too.

pub mod actions;
pub mod delete;
pub mod entity;
pub mod field;
pub mod form;
pub mod inputs;
pub mod instance;
pub mod leave;
pub mod list;
pub mod notify;
pub mod table;
pub mod tabs;
pub mod texts;
pub mod views;

pub use actions::{ActionId, CrudActionHandle, CrudActionsState, use_crud_actions_state};
pub use delete::{CrudDeleteState, use_crud_delete};
pub use entity::{CrudEntityHandle, use_crud_entity};
pub use field::{
    CrudFieldBinding, CrudFieldInputError, CrudFieldMap, CrudFieldState, CrudRowFields,
    ReactiveField, use_crud_field, use_crud_field_binding, use_crud_row_fields,
};
pub use form::{
    CrudEntityLoadStatus, CrudFormHandle, CrudFormState, CrudSaveFollowUp, CrudSaveNotifications,
    UseCrudCreateFormInput, UseCrudCreateFormReturn, UseCrudEditFormInput, UseCrudEditFormReturn,
    UseCrudReadReturn, use_crud_create_form, use_crud_edit_form, use_crud_form, use_crud_read,
};
pub use inputs::{
    CrudInputKind, CrudInputProps, CrudNumber, CrudNumberKind, CrudTextCodec, CrudValueFormatter,
    CrudValueParser, UseCrudNumberInputReturn, UseCrudTextInputReturn, UseCrudToggleInputReturn,
    format_crud_value, use_crud_number_input, use_crud_text_input, use_crud_toggle_input,
};
pub use instance::{use_crud_instance, use_crud_navigation};
pub use leave::{CrudLeaveConfirmation, use_crud_leave_confirmation};
pub use list::{
    CrudListLoadStatus, CrudListState, CrudOrderingState, CrudPageRange, CrudPaginationState,
    CrudSelectionState, use_crud_list, use_crud_list_state,
};
pub use notify::{
    CrudNotification, CrudNotificationKind, CrudNotificationOrigin, CrudNotifier,
    provide_crud_notifier, use_crud_notifier,
};
pub use table::{
    CrudTableData, UseCrudTableInput, UseCrudTableReturn, use_crud_table, use_crud_table_data,
    use_crud_table_row,
};
pub use tabs::{UseCrudTabSelectionReturn, use_crud_tab_selection};
pub use texts::{provide_crud_texts, use_crud_texts};
pub use views::provide_crud_view_registry;

// Hooks load data in client mode only, so their tests run without the `ssr` feature.
#[cfg(all(test, not(target_family = "wasm"), not(feature = "ssr")))]
mod tests;
