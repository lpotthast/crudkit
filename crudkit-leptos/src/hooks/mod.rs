//! Hooks exposing CrudKit's behavior independently of its built-in markup.
//!
//! State hooks (`use_crud_*`) return `Copy` handles to reactive state owned by the surrounding
//! [`crate::components::instance::CrudInstance`] or by the calling view. They render nothing, so
//! applications can build fully custom views on top of them.

pub mod actions;
pub mod delete;
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

pub use actions::{ActionId, CrudActionHandle, CrudActionsState, use_crud_actions};
pub use delete::{CrudDeleteState, use_crud_delete};
pub use field::{
    CrudFieldInputError, CrudFieldMap, CrudFieldState, CrudRowFields, ReactiveField,
    use_crud_field, use_crud_row_fields,
};
pub use form::{
    CrudCreateActions, CrudEntityStatus, CrudFormState, CrudSaveFollowUp, UseCrudCreateFormInput,
    UseCrudCreateFormReturn, UseCrudEditFormInput, UseCrudEditFormReturn, UseCrudReadReturn,
    use_crud_create_form, use_crud_edit_form, use_crud_read,
};
pub use inputs::{
    CrudInputKind, CrudInputProps, CrudNumber, CrudNumberKind, CrudTextCodec, CrudValueFormatter,
    CrudValueParser, UseCrudNumberInputReturn, UseCrudTextInputReturn, UseCrudToggleInputReturn,
    use_crud_number_input, use_crud_text_input, use_crud_toggle_input,
};
pub use instance::use_crud_instance;
pub use leave::{CrudLeaveConfirmation, use_crud_leave_confirmation};
pub use list::{
    CrudListState, CrudOrderingState, CrudPaginationState, CrudSelectionState, use_crud_list,
};
pub use notify::{
    CrudNotification, CrudNotificationId, CrudNotificationKind, CrudNotificationQueue,
    CrudNotifier, provide_crud_notifier, use_crud_notifier,
};
pub use table::{
    CrudTableColumn, CrudTableData, UseCrudTableInput, UseCrudTableReturn, use_crud_table,
    use_crud_table_data,
};
pub use tabs::{UseCrudTabSelectionReturn, use_crud_tab_selection};
pub use texts::{provide_crud_texts, use_crud_texts};
pub use views::provide_crud_view_registry;

#[cfg(all(test, not(target_family = "wasm")))]
mod tests;
