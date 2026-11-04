//! The texts that CrudKit shows or announces, and their translation.

use std::borrow::Cow;

/// Defines [`CrudText`] from one list of texts: the enum, the ID and the arguments of each text,
/// and the [catalog](CrudText::catalog) of every text. Each text is a variant whose fields are its
/// arguments, each with an example value after `=`, followed by `=>` and the text's ID. Generating
/// the catalog from the list keeps it complete.
macro_rules! crud_texts {
    (
        $(#[$meta:meta])*
        pub enum CrudText<$lt:lifetime> {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident $({
                    $(
                        $(#[$field_meta:meta])*
                        $field:ident: $ty:ty = $example:expr
                    ),* $(,)?
                })? => $id:literal
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum CrudText<$lt> {
            $(
                $(#[$variant_meta])*
                $variant $({ $( $(#[$field_meta])* $field: $ty ),* })?,
            )*
        }

        impl<$lt> CrudText<$lt> {
            /// Returns the ID of this text: `crudkit-` followed by the name of its variant in kebab
            /// case.
            #[must_use]
            pub fn id(&self) -> &'static str {
                match self {
                    $(Self::$variant { .. } => $id,)*
                }
            }

            /// Returns the arguments of this text: the fields of its variant, by their names, in
            /// the order of the fields. Empty for a text without arguments.
            #[must_use]
            pub fn args(&self) -> Vec<(&'static str, CrudTextArg<$lt>)> {
                match *self {
                    $(
                        Self::$variant $({ $($field),* })? => vec![
                            $($((stringify!($field), CrudTextArg::from($field))),*)?
                        ],
                    )*
                }
            }
        }

        impl CrudText<'static> {
            /// Returns every text once, in the order of the variants, with example arguments. A
            /// product checks its translations against it: that each translation names only these
            /// IDs, and only the arguments of its text.
            #[must_use]
            pub fn catalog() -> &'static [CrudText<'static>] {
                &[$(Self::$variant $({ $($field: $example),* })?,)*]
            }
        }
    };
}

crud_texts! {
    /// A text that CrudKit shows or announces: the default content of an atom, an accessible name,
    /// the title or message of a notification, or the explanation of an error.
    ///
    /// Each text has a stable [ID](Self::id), named [arguments](Self::args), and an English
    /// default ([`Self::english`]). A [`CrudTranslator`] looks texts up in the user's language.
    /// Without one, CrudKit shows the English defaults. [`Self::catalog`] lists every text.
    ///
    /// The ID of a text is `crudkit-` followed by the name of its variant in kebab case, such as
    /// `crudkit-form-save` for [`Self::FormSave`]. The arguments of a text are the fields of its
    /// variant, by their names, such as `count` for [`Self::SelectionCount`]. IDs and argument
    /// names are valid identifiers of Fluent messages and variables, so a Fluent resource can name
    /// them directly. They change only in a breaking release.
    ///
    /// The documentation of each variant names where the text shows, and its English default. A
    /// text that takes a count chooses its plural form itself, and so does a translation. CrudKit
    /// joins the sentences of one message, such as those of a mass deletion's outcome, with a
    /// space.
    ///
    /// The enum is exhaustive on purpose: a new text is a breaking change, so that a translator
    /// written in Rust, which matches the variants without a wildcard arm, learns of every new
    /// text when it compiles.
    pub enum CrudText<'a> {
        /// The status of a list or an entity that loads: "Loading…".
        StatusLoading => "crudkit-status-loading",
        /// The status of a list without entities: "No data".
        StatusEmpty => "crudkit-status-empty",
        /// The status of an entity that does not exist: "This entry does not exist."
        StatusNotFound => "crudkit-status-not-found",
        /// The status of a list or an entity that could not be loaded: "Data unavailable:
        /// {error}".
        StatusFailed {
            /// Why loading failed, e.g. the text of a request error
            /// ([`CrudTexts::request_error`](crate::hooks::CrudTexts::request_error)).
            error: &'a str = "Network error",
        } => "crudkit-status-failed",
        /// The header of a table's actions column: "Actions".
        TableActions => "crudkit-table-actions",
        /// The accessible name of the checkbox that selects all rows: "Select all".
        TableSelectAll => "crudkit-table-select-all",
        /// The accessible name of a row's selection checkbox: "Select".
        TableSelectRow => "crudkit-table-select-row",
        /// The number of selected entities: "{count} selected".
        SelectionCount {
            /// The number of selected entities.
            count: u64 = 3,
        } => "crudkit-selection-count",
        /// Deselects all entities: "Clear selection".
        SelectionClear => "crudkit-selection-clear",
        /// Deletes the selected entities: "Delete selection".
        SelectionDelete => "crudkit-selection-delete",
        /// Resets an instance to its configuration: "Reset".
        InstanceReset => "crudkit-instance-reset",
        /// The accessible name of a list's pagination: "Pages".
        PaginationLabel => "crudkit-pagination-label",
        /// Opens the previous page: "Previous page".
        PaginationPrevious => "crudkit-pagination-previous",
        /// Opens the next page: "Next page".
        PaginationNext => "crudkit-pagination-next",
        /// The accessible name of a page's button: "Page {page}".
        PaginationPage {
            /// The number of the page, counted from one.
            page: u64 = 2,
        } => "crudkit-pagination-page",
        /// The label of the page size selection: "Entries per page".
        PaginationPageSize => "crudkit-pagination-page-size",
        /// Opens the create view: "New".
        EntityCreate => "crudkit-entity-create",
        /// Opens the read view of an entity: "View".
        EntityRead => "crudkit-entity-read",
        /// Opens the edit view of an entity: "Edit".
        EntityEdit => "crudkit-entity-edit",
        /// Deletes an entity, also in the confirmation dialogs of deletions: "Delete".
        EntityDelete => "crudkit-entity-delete",
        /// Saves a form: "Save".
        FormSave => "crudkit-form-save",
        /// Saves a form and returns: "Save and return".
        FormSaveAndReturn => "crudkit-form-save-and-return",
        /// Saves a form and opens a fresh create view: "Save and new".
        FormSaveAndNew => "crudkit-form-save-and-new",
        /// Returns from the current view: "Return".
        FormReturn => "crudkit-form-return",
        /// Reloads the entity of a form to show its current state, keeping the input: "Show
        /// current state".
        FormReload => "crudkit-form-reload",
        /// Explains that saving was refused, because someone else changed the entity since the
        /// form loaded it: "Someone else changed this entry in the meantime. Show its current
        /// state, then save again. Your input is kept."
        FormOutdated => "crudkit-form-outdated",
        /// Explains that the user's changes cannot be applied to the current version of the
        /// entity: "Your changes cannot be applied to the current version of this entry, which
        /// someone else changed. Leave the entry to discard your changes, then open it again."
        FormRebaseFailed => "crudkit-form-rebase-failed",
        /// Explains that someone else changed fields that the user changed too, which a reload
        /// found: "Someone else also changed fields that you changed. Check them, then save
        /// again."
        FormConflicts => "crudkit-form-conflicts",
        /// Explains that someone else changed a value that the user changed too: "Someone else
        /// changed this to “{value}” in the meantime."
        FormChangedMeanwhile {
            /// The value that someone else set: a string as it is, the value of a boolean field
            /// as [`Self::ValueYes`] or [`Self::ValueNo`], as the field shows it, and any other
            /// value as JSON.
            value: &'a str = "blue",
        } => "crudkit-form-changed-meanwhile",
        /// Explains that someone else removed a value that the user changed: "Someone else
        /// removed this in the meantime."
        FormRemovedMeanwhile => "crudkit-form-removed-meanwhile",
        /// A problem or a conflict at a place of a form: "{location}: {message}".
        FormProblem {
            /// Where the problem is: the path of a member below a field, such as `0/count`, or in
            /// the form's error summary the field's label and that path, such as
            /// `Players/0/count`.
            location: &'a str = "0/count",
            /// What the problem is, e.g. the message of the data source.
            message: &'a str = "must be positive",
        } => "crudkit-form-problem",
        /// The title of the notification after an entity was created, also when the data source
        /// still sets it up: "Entry created".
        SaveCreatedTitle => "crudkit-save-created-title",
        /// The message after an entity was created: "The entry was created."
        SaveCreated => "crudkit-save-created",
        /// The message after the data source accepted the creation of an entity, which it still
        /// sets up: "The entry was created and is being set up."
        SaveCreateAccepted => "crudkit-save-create-accepted",
        /// The title of the notification after creating an entity failed: "Entry not created".
        /// The message is the text of the error.
        SaveCreateFailedTitle => "crudkit-save-create-failed-title",
        /// The title of the notification after a change was saved: "Changes saved".
        SaveUpdatedTitle => "crudkit-save-updated-title",
        /// The message after a change was saved: "The changes were saved."
        SaveUpdated => "crudkit-save-updated",
        /// The title of the notification after the data source accepted a change, which it still
        /// applies: "Changes accepted".
        SaveUpdateAcceptedTitle => "crudkit-save-update-accepted-title",
        /// The message after the data source accepted a change, which it still applies: "The
        /// changes were accepted and are being applied."
        SaveUpdateAccepted => "crudkit-save-update-accepted",
        /// The title of the notification after a change was refused, because someone else changed
        /// the entity since the form loaded it: "Changes not saved". The message is
        /// [`Self::FormOutdated`].
        SaveOutdatedTitle => "crudkit-save-outdated-title",
        /// The title of the notification after saving a change failed: "Changes not saved". The
        /// message is the text of the error.
        SaveUpdateFailedTitle => "crudkit-save-update-failed-title",
        /// The title of the notification after entities were deleted: "Entry deleted" or "Entries
        /// deleted".
        DeleteCompletedTitle {
            /// The number of deleted entities, including those still being deleted.
            count: u64 = 3,
        } => "crudkit-delete-completed-title",
        /// The message after entities were deleted: "1 entry deleted." or "{count} entries
        /// deleted."
        DeleteCompleted {
            /// The number of deleted entities.
            count: u64 = 3,
        } => "crudkit-delete-completed",
        /// The title of the notification after the data source accepted the deletion of entities,
        /// which it still deletes: "Entry being deleted" or "Entries being deleted".
        DeleteAcceptedTitle {
            /// The number of entities being deleted.
            count: u64 = 3,
        } => "crudkit-delete-accepted-title",
        /// The message after the data source accepted the deletion of entities, which it still
        /// deletes: "1 entry is being deleted." or "{count} entries are being deleted."
        DeleteAccepted {
            /// The number of entities being deleted.
            count: u64 = 3,
        } => "crudkit-delete-accepted",
        /// The title of the notification after the data source refused to delete an entity, e.g.
        /// for lack of permission, because of its rules, or because someone else changed the
        /// entity: "Entry not deleted". The message is the text of the error, or
        /// [`Self::DeleteOutdated`].
        DeleteRefusedTitle => "crudkit-delete-refused-title",
        /// The message after a deletion was refused, because someone else changed the entity since
        /// the user saw it: "Someone else changed this entry in the meantime. Check the changes,
        /// then delete it again."
        DeleteOutdated => "crudkit-delete-outdated",
        /// The title of the notification after deleting entities failed: "Entry not deleted" or
        /// "Entries not deleted". The message is the text of the error, or the sentences of a mass
        /// deletion that deleted nothing.
        DeleteFailedTitle {
            /// The number of entities that were to be deleted.
            count: u64 = 3,
        } => "crudkit-delete-failed-title",
        /// The title of the notification after a mass deletion that deleted some of its entities:
        /// "Some entries not deleted".
        DeleteManyPartialTitle => "crudkit-delete-many-partial-title",
        /// The first sentence after a mass deletion that deleted some of its entities: "1 entry
        /// deleted." or "{count} entries deleted."
        DeleteManyPartial {
            /// The number of deleted entities.
            count: u64 = 3,
        } => "crudkit-delete-many-partial",
        /// The first sentence after a mass deletion that deleted nothing: "No entries deleted."
        DeleteManyNone => "crudkit-delete-many-none",
        /// The sentence about the deletions of a mass deletion that the data source refused:
        /// "{count} refused."
        DeleteManyAborted {
            /// The number of refused deletions.
            count: u64 = 3,
        } => "crudkit-delete-many-aborted",
        /// The sentence about the deletions of a mass deletion that validation prevented: "1
        /// validation error." or "{count} validation errors."
        DeleteManyInvalid {
            /// The number of prevented deletions.
            count: u64 = 3,
        } => "crudkit-delete-many-invalid",
        /// The sentence about the deletions of a mass deletion that failed: "1 error." or
        /// "{count} errors."
        DeleteManyFailed {
            /// The number of failed deletions.
            count: u64 = 3,
        } => "crudkit-delete-many-failed",
        /// The sentence about the entities of a mass deletion that a failure left untried:
        /// "{count} not attempted."
        DeleteManySkipped {
            /// The number of untried entities.
            count: u64 = 3,
        } => "crudkit-delete-many-skipped",
        /// The title of the dialog confirming a deletion: "Delete entry".
        DeleteDialogTitle => "crudkit-delete-dialog-title",
        /// The message of the dialog confirming a deletion: "Are you sure? This entry cannot be
        /// restored."
        DeleteDialogMessage => "crudkit-delete-dialog-message",
        /// The title of the dialog confirming a mass deletion: "Delete 1 entry" or "Delete {count}
        /// entries".
        DeleteManyDialogTitle {
            /// The number of entities to delete.
            count: u64 = 3,
        } => "crudkit-delete-many-dialog-title",
        /// The message of the dialog confirming a mass deletion: "Are you sure? This entry cannot
        /// be restored." or "Are you sure? These {count} entries cannot be restored."
        DeleteManyDialogMessage {
            /// The number of entities to delete.
            count: u64 = 3,
        } => "crudkit-delete-many-dialog-message",
        /// The title of the dialog confirming to leave unsaved changes: "Unsaved changes".
        LeaveDialogTitle => "crudkit-leave-dialog-title",
        /// The message of the dialog confirming to leave unsaved changes: "You have not saved your
        /// changes. Do you really want to leave? Unsaved changes are lost."
        LeaveDialogMessage => "crudkit-leave-dialog-message",
        /// Confirms leaving and discarding the changes: "Leave".
        LeaveDialogConfirm => "crudkit-leave-dialog-confirm",
        /// Cancels a confirmation dialog: "Cancel".
        DialogCancel => "crudkit-dialog-cancel",
        /// A `true` value shown as text: "Yes".
        ValueYes => "crudkit-value-yes",
        /// A `false` value shown as text: "No".
        ValueNo => "crudkit-value-no",
        /// Rejects the input of a UUID field: "Enter a UUID, such as
        /// 67e55044-10b1-426f-9247-bb680e5fe0c8."
        InputInvalidUuid => "crudkit-input-invalid-uuid",
        /// Rejects the input of a date-time field: "Enter a date and time in the form
        /// YYYY-MM-DDTHH:MM or YYYY-MM-DDTHH:MM:SS."
        InputInvalidDateTime => "crudkit-input-invalid-date-time",
        /// Rejects the input of a date-time field with offset: "Enter a date and time with an
        /// offset, such as 2026-11-04T09:30:00+01:00."
        InputInvalidOffsetDateTime => "crudkit-input-invalid-offset-date-time",
        /// Rejects the input of a duration field: "Enter a duration in the form HH:MM, HH:MM:SS,
        /// or HH:MM:SS.cc."
        InputInvalidDuration => "crudkit-input-invalid-duration",
        /// Rejects the input of a JSON field that is not JSON: "Invalid JSON at line {line},
        /// column {column}."
        InputInvalidJson {
            /// The line of the first error, counted from one.
            line: u64 = 2,
            /// The column of the first error in characters, counted from one.
            column: u64 = 7,
        } => "crudkit-input-invalid-json",
        /// Rejects the input of a JSON field that ends before its JSON is complete: "The JSON ends
        /// too early."
        InputIncompleteJson => "crudkit-input-incomplete-json",
        /// A configuration error: a field of a custom type has no registered renderer: "No
        /// renderer is registered for the field “{field}”."
        FieldNoRenderer {
            /// The name of the field.
            field: &'a str = "tags",
        } => "crudkit-field-no-renderer",
        /// A configuration error: a form shows a field that its model does not have: "The field
        /// “{field}” is not part of this form."
        FieldUnknown {
            /// The name of the field.
            field: &'a str = "tags",
        } => "crudkit-field-unknown",
        /// A configuration error: a view has no registered renderer: "No renderer is registered
        /// for the view “{view}”."
        ViewNotRegistered {
            /// The name of the view.
            view: &'a str = "app.audit",
        } => "crudkit-view-not-registered",
        /// A configuration error: a standard view without an entity was opened with one: "The
        /// view “{view}” does not accept an entity subject."
        ViewRejectsSubject {
            /// The name of the view.
            view: &'a str = "table",
        } => "crudkit-view-rejects-subject",
        /// A configuration error: a standard view of an entity was opened without one: "The view
        /// “{view}” requires an entity subject."
        ViewNeedsSubject {
            /// The name of the view.
            view: &'a str = "edit",
        } => "crudkit-view-needs-subject",
        /// A configuration error: a standard view was opened with a payload: "The view “{view}”
        /// does not accept a payload."
        ViewRejectsPayload {
            /// The name of the view.
            view: &'a str = "table",
        } => "crudkit-view-rejects-payload",
        /// The data source rejected a request as malformed (`RequestError::BadRequest`): "Request
        /// rejected".
        ErrorBadRequest => "crudkit-error-bad-request",
        /// The request needs the user to sign in (`RequestError::Unauthorized`): "Not signed in".
        ErrorUnauthorized => "crudkit-error-unauthorized",
        /// The user lacks permission (`RequestError::Forbidden`): "Permission denied".
        ErrorForbidden => "crudkit-error-forbidden",
        /// The entity does not exist (`RequestError::NotFound`): "Not found".
        ErrorNotFound => "crudkit-error-not-found",
        /// The data source's rules do not allow the operation
        /// (`RequestError::UnprocessableEntity`): "Not possible".
        ErrorUnprocessable => "crudkit-error-unprocessable",
        /// Critical validation errors prevent the operation
        /// (`RequestError::CriticalValidationErrors`): "Validation failed".
        ErrorValidation => "crudkit-error-validation",
        /// Someone else changed the entity since the version that the request is based on
        /// (`RequestError::Outdated`): "Changed by someone else".
        ErrorOutdated => "crudkit-error-outdated",
        /// Problems with fields refused the request (`RequestError::InvalidFields`): "Invalid
        /// input".
        ErrorInvalidFields => "crudkit-error-invalid-fields",
        /// The data source failed (`RequestError::InternalServerError`): "Server error".
        ErrorServer => "crudkit-error-server",
        /// The client could not build the request, so it sent nothing
        /// (`RequestError::InvalidRequest`): "Request not sent". Its message is a technical
        /// detail in English, see [`Self::ErrorDetail`].
        ErrorInvalidRequest => "crudkit-error-invalid-request",
        /// The request failed in transport, or the response had an unexpected status
        /// (`RequestError::Request`): "Network error". Its message is a technical detail in
        /// English, see [`Self::ErrorDetail`].
        ErrorNetwork => "crudkit-error-network",
        /// The response could not be read (`RequestError::Deserialize`): "Invalid response". Its
        /// message is a technical detail in English, see [`Self::ErrorDetail`].
        ErrorInvalidResponse => "crudkit-error-invalid-response",
        /// A request error with the message of the data source, which is meant for the user:
        /// "{error}: {message}". An error without a message shows its text alone.
        ErrorMessage {
            /// The text of the error's kind, e.g. [`Self::ErrorForbidden`].
            error: &'a str = "Permission denied",
            /// The message of the data source, e.g. why it refused the request.
            message: &'a str = "The club still has players.",
        } => "crudkit-error-message",
        /// A request error with a technical detail: "{error}: {detail}". The detail is English
        /// text for diagnosing the error, such as the message of a network library, so a
        /// translation may leave it out. An error without a detail shows its text alone.
        ErrorDetail {
            /// The text of the error's kind, e.g. [`Self::ErrorNetwork`].
            error: &'a str = "Network error",
            /// The technical detail, in English.
            detail: &'a str = "connection refused",
        } => "crudkit-error-detail",
    }
}

/// The value of an argument of a [`CrudText`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrudTextArg<'a> {
    /// A number, e.g. a count, from which a translation chooses a plural form.
    Number(u64),
    /// Text, e.g. the name of a field or a message of the data source.
    String(&'a str),
}

impl From<u64> for CrudTextArg<'_> {
    fn from(number: u64) -> Self {
        Self::Number(number)
    }
}

impl<'a> From<&'a str> for CrudTextArg<'a> {
    fn from(string: &'a str) -> Self {
        Self::String(string)
    }
}

impl CrudText<'_> {
    /// Returns the English default of this text, with its arguments.
    #[must_use]
    // One line or a few per text, so the function grows with the texts.
    #[allow(clippy::too_many_lines)]
    pub fn english(&self) -> Cow<'static, str> {
        let entries = |count: u64| match count {
            1 => "1 entry".to_owned(),
            _ => format!("{count} entries"),
        };
        let entry_or_entries = |count: u64| if count == 1 { "Entry" } else { "Entries" };
        match *self {
            Self::StatusLoading => "Loading…".into(),
            Self::StatusEmpty => "No data".into(),
            Self::StatusNotFound => "This entry does not exist.".into(),
            Self::StatusFailed { error } => format!("Data unavailable: {error}").into(),
            Self::TableActions => "Actions".into(),
            Self::TableSelectAll => "Select all".into(),
            Self::TableSelectRow => "Select".into(),
            Self::SelectionCount { count } => format!("{count} selected").into(),
            Self::SelectionClear => "Clear selection".into(),
            Self::SelectionDelete => "Delete selection".into(),
            Self::InstanceReset => "Reset".into(),
            Self::PaginationLabel => "Pages".into(),
            Self::PaginationPrevious => "Previous page".into(),
            Self::PaginationNext => "Next page".into(),
            Self::PaginationPage { page } => format!("Page {page}").into(),
            Self::PaginationPageSize => "Entries per page".into(),
            Self::EntityCreate => "New".into(),
            Self::EntityRead => "View".into(),
            Self::EntityEdit => "Edit".into(),
            Self::EntityDelete => "Delete".into(),
            Self::FormSave => "Save".into(),
            Self::FormSaveAndReturn => "Save and return".into(),
            Self::FormSaveAndNew => "Save and new".into(),
            Self::FormReturn => "Return".into(),
            Self::FormReload => "Show current state".into(),
            Self::FormOutdated => "Someone else changed this entry in the meantime. Show its \
                                   current state, then save again. Your input is kept."
                .into(),
            Self::FormRebaseFailed => "Your changes cannot be applied to the current version of \
                                       this entry, which someone else changed. Leave the entry \
                                       to discard your changes, then open it again."
                .into(),
            Self::FormConflicts => {
                "Someone else also changed fields that you changed. Check them, then save again."
                    .into()
            }
            Self::FormChangedMeanwhile { value } => {
                format!("Someone else changed this to “{value}” in the meantime.").into()
            }
            Self::FormRemovedMeanwhile => "Someone else removed this in the meantime.".into(),
            Self::FormProblem { location, message } => format!("{location}: {message}").into(),
            Self::SaveCreatedTitle => "Entry created".into(),
            Self::SaveCreated => "The entry was created.".into(),
            Self::SaveCreateAccepted => "The entry was created and is being set up.".into(),
            Self::SaveCreateFailedTitle => "Entry not created".into(),
            Self::SaveUpdatedTitle => "Changes saved".into(),
            Self::SaveUpdated => "The changes were saved.".into(),
            Self::SaveUpdateAcceptedTitle => "Changes accepted".into(),
            Self::SaveUpdateAccepted => "The changes were accepted and are being applied.".into(),
            Self::SaveOutdatedTitle | Self::SaveUpdateFailedTitle => "Changes not saved".into(),
            Self::DeleteCompletedTitle { count } => {
                format!("{} deleted", entry_or_entries(count)).into()
            }
            Self::DeleteCompleted { count } | Self::DeleteManyPartial { count } => {
                format!("{} deleted.", entries(count)).into()
            }
            Self::DeleteAcceptedTitle { count } => {
                format!("{} being deleted", entry_or_entries(count)).into()
            }
            Self::DeleteAccepted { count } => match count {
                1 => "1 entry is being deleted.".into(),
                _ => format!("{count} entries are being deleted.").into(),
            },
            Self::DeleteRefusedTitle => "Entry not deleted".into(),
            Self::DeleteOutdated => "Someone else changed this entry in the meantime. Check the \
                                     changes, then delete it again."
                .into(),
            Self::DeleteFailedTitle { count } => {
                format!("{} not deleted", entry_or_entries(count)).into()
            }
            Self::DeleteManyPartialTitle => "Some entries not deleted".into(),
            Self::DeleteManyNone => "No entries deleted.".into(),
            Self::DeleteManyAborted { count } => format!("{count} refused.").into(),
            Self::DeleteManyInvalid { count } => match count {
                1 => "1 validation error.".into(),
                _ => format!("{count} validation errors.").into(),
            },
            Self::DeleteManyFailed { count } => match count {
                1 => "1 error.".into(),
                _ => format!("{count} errors.").into(),
            },
            Self::DeleteManySkipped { count } => format!("{count} not attempted.").into(),
            Self::DeleteDialogTitle => "Delete entry".into(),
            Self::DeleteDialogMessage | Self::DeleteManyDialogMessage { count: 1 } => {
                "Are you sure? This entry cannot be restored.".into()
            }
            Self::DeleteManyDialogTitle { count } => format!("Delete {}", entries(count)).into(),
            Self::DeleteManyDialogMessage { count } => {
                format!("Are you sure? These {count} entries cannot be restored.").into()
            }
            Self::LeaveDialogTitle => "Unsaved changes".into(),
            Self::LeaveDialogMessage => "You have not saved your changes. Do you really want to \
                                         leave? Unsaved changes are lost."
                .into(),
            Self::LeaveDialogConfirm => "Leave".into(),
            Self::DialogCancel => "Cancel".into(),
            Self::ValueYes => "Yes".into(),
            Self::ValueNo => "No".into(),
            Self::InputInvalidUuid => {
                "Enter a UUID, such as 67e55044-10b1-426f-9247-bb680e5fe0c8.".into()
            }
            Self::InputInvalidDateTime => "Enter a date and time in the form YYYY-MM-DDTHH:MM or \
                                           YYYY-MM-DDTHH:MM:SS."
                .into(),
            Self::InputInvalidOffsetDateTime => {
                "Enter a date and time with an offset, such as 2026-11-04T09:30:00+01:00.".into()
            }
            Self::InputInvalidDuration => {
                "Enter a duration in the form HH:MM, HH:MM:SS, or HH:MM:SS.cc.".into()
            }
            Self::InputInvalidJson { line, column } => {
                format!("Invalid JSON at line {line}, column {column}.").into()
            }
            Self::InputIncompleteJson => "The JSON ends too early.".into(),
            Self::FieldNoRenderer { field } => {
                format!("No renderer is registered for the field “{field}”.").into()
            }
            Self::FieldUnknown { field } => {
                format!("The field “{field}” is not part of this form.").into()
            }
            Self::ViewNotRegistered { view } => {
                format!("No renderer is registered for the view “{view}”.").into()
            }
            Self::ViewRejectsSubject { view } => {
                format!("The view “{view}” does not accept an entity subject.").into()
            }
            Self::ViewNeedsSubject { view } => {
                format!("The view “{view}” requires an entity subject.").into()
            }
            Self::ViewRejectsPayload { view } => {
                format!("The view “{view}” does not accept a payload.").into()
            }
            Self::ErrorBadRequest => "Request rejected".into(),
            Self::ErrorUnauthorized => "Not signed in".into(),
            Self::ErrorForbidden => "Permission denied".into(),
            Self::ErrorNotFound => "Not found".into(),
            Self::ErrorUnprocessable => "Not possible".into(),
            Self::ErrorValidation => "Validation failed".into(),
            Self::ErrorOutdated => "Changed by someone else".into(),
            Self::ErrorInvalidFields => "Invalid input".into(),
            Self::ErrorServer => "Server error".into(),
            Self::ErrorInvalidRequest => "Request not sent".into(),
            Self::ErrorNetwork => "Network error".into(),
            Self::ErrorInvalidResponse => "Invalid response".into(),
            Self::ErrorMessage { error, message } => format!("{error}: {message}").into(),
            Self::ErrorDetail { error, detail } => format!("{error}: {detail}").into(),
        }
    }
}

/// Looks CrudKit's texts up in the user's language.
///
/// Provide a translator once, at the root of the application, with
/// [`provide_crud_translator`](crate::hooks::provide_crud_translator). Without one, CrudKit shows
/// the English defaults ([`CrudText::english`]). A translator may translate some texts only:
/// CrudKit shows the English default of every text that the translator returns `None` for. A
/// closure `Fn(&CrudText<'_>) -> Option<String>` is a translator too.
///
/// A translator backed by a library of translations looks a text up by its [ID](CrudText::id)
/// and passes its [arguments](CrudText::args), keeping numbers as numbers, so that the
/// translation chooses its plural forms.
///
/// CrudKit looks texts up where it renders them. A translator that reads reactive state, such as
/// a signal holding the user's locale, therefore switches every rendered text when that state
/// changes. CrudKit also looks texts up without tracking, e.g. for a notification after a request
/// completed, possibly after the view that sent it is gone. So a translator reads state that
/// lives as long as the translator, such as the application's locale.
///
/// Server rendering and hydration render the same texts when the server and the browser provide
/// translators for the same locale.
pub trait CrudTranslator: Send + Sync {
    /// Returns `text` in the user's language, or `None` to show its English default.
    fn translate(&self, text: &CrudText<'_>) -> Option<String>;
}

impl<F> CrudTranslator for F
where
    F: Fn(&CrudText<'_>) -> Option<String> + Send + Sync,
{
    fn translate(&self, text: &CrudText<'_>) -> Option<String> {
        self(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;
    use std::collections::HashSet;

    /// Returns `name` (`UpperCamelCase`) in kebab case.
    fn kebab_case(name: &str) -> String {
        let mut kebab = String::new();
        for character in name.chars() {
            if character.is_ascii_uppercase() {
                if !kebab.is_empty() {
                    kebab.push('-');
                }
                kebab.push(character.to_ascii_lowercase());
            } else {
                kebab.push(character);
            }
        }
        kebab
    }

    /// Returns the name of the variant of `text`, read from its `Debug` output.
    fn variant_name(text: &CrudText<'_>) -> String {
        format!("{text:?}")
            .split([' ', '{'])
            .next()
            .unwrap_or_default()
            .to_owned()
    }

    /// Returns the names of the fields of the variant of `text`, read from its `Debug` output. The
    /// examples of the catalog contain no `{`, `,`, or `:`, so that they do not end a name.
    fn field_names(text: &CrudText<'_>) -> Vec<String> {
        format!("{text:?}")
            .split(['{', ','])
            .skip(1)
            .filter_map(|field| field.split(':').next())
            .map(|name| name.trim().to_owned())
            .collect()
    }

    /// Whether `name` is an identifier of Fluent: `[a-zA-Z][a-zA-Z0-9_-]*`.
    fn is_fluent_identifier(name: &str) -> bool {
        let mut characters = name.chars();
        characters
            .next()
            .is_some_and(|first| first.is_ascii_alphabetic())
            && characters.all(|it| it.is_ascii_alphanumeric() || it == '_' || it == '-')
    }

    #[test]
    fn the_catalog_lists_every_text_once() {
        let variants = CrudText::catalog()
            .iter()
            .map(variant_name)
            .collect::<Vec<_>>();

        assert_that!(variants.iter().collect::<HashSet<_>>().len()).is_equal_to(variants.len());
        assert_that!(variants.first().map(String::as_str)).is_equal_to(Some("StatusLoading"));
        assert_that!(variants.last().map(String::as_str)).is_equal_to(Some("ErrorDetail"));
    }

    #[test]
    fn ids_name_their_variant_in_kebab_case_and_are_unique() {
        let ids = CrudText::catalog()
            .iter()
            .map(CrudText::id)
            .collect::<Vec<_>>();

        for (text, id) in CrudText::catalog().iter().zip(&ids) {
            let expected = format!("crudkit-{}", kebab_case(&variant_name(text)));
            assert_that!(*id).is_equal_to(expected.as_str());
            assert_that!(is_fluent_identifier(id)).is_true();
        }
        assert_that!(ids.iter().collect::<HashSet<_>>().len()).is_equal_to(ids.len());
    }

    #[test]
    fn arguments_are_the_fields_of_their_variant() {
        for text in CrudText::catalog() {
            let names = text
                .args()
                .into_iter()
                .map(|(name, _)| name.to_owned())
                .collect::<Vec<_>>();

            assert_that!(names.iter().all(|name| is_fluent_identifier(name))).is_true();
            assert_that!(names).is_equal_to(field_names(text));
        }
    }

    #[test]
    fn english_defaults_contain_their_arguments() {
        for text in CrudText::catalog() {
            let english = text.english();

            for (_, value) in text.args() {
                let value = match value {
                    // A title's count only chooses its plural form, e.g. "Entries deleted".
                    CrudTextArg::Number(_) if text.id().ends_with("-title") => continue,
                    CrudTextArg::Number(number) => number.to_string(),
                    CrudTextArg::String(string) => string.to_owned(),
                };
                assert_that!(english.as_ref()).contains(value.as_str());
            }
        }
    }

    /// Every text that takes a count, with its English defaults for the counts 0, 1, and 2.
    type CountTexts = [(fn(u64) -> CrudText<'static>, [&'static str; 3])];

    const COUNT_TEXTS: &CountTexts = &[
        (
            |count| CrudText::SelectionCount { count },
            ["0 selected", "1 selected", "2 selected"],
        ),
        (
            |count| CrudText::DeleteCompletedTitle { count },
            ["Entries deleted", "Entry deleted", "Entries deleted"],
        ),
        (
            |count| CrudText::DeleteCompleted { count },
            [
                "0 entries deleted.",
                "1 entry deleted.",
                "2 entries deleted.",
            ],
        ),
        (
            |count| CrudText::DeleteAcceptedTitle { count },
            [
                "Entries being deleted",
                "Entry being deleted",
                "Entries being deleted",
            ],
        ),
        (
            |count| CrudText::DeleteAccepted { count },
            [
                "0 entries are being deleted.",
                "1 entry is being deleted.",
                "2 entries are being deleted.",
            ],
        ),
        (
            |count| CrudText::DeleteFailedTitle { count },
            [
                "Entries not deleted",
                "Entry not deleted",
                "Entries not deleted",
            ],
        ),
        (
            |count| CrudText::DeleteManyPartial { count },
            [
                "0 entries deleted.",
                "1 entry deleted.",
                "2 entries deleted.",
            ],
        ),
        (
            |count| CrudText::DeleteManyAborted { count },
            ["0 refused.", "1 refused.", "2 refused."],
        ),
        (
            |count| CrudText::DeleteManyInvalid { count },
            [
                "0 validation errors.",
                "1 validation error.",
                "2 validation errors.",
            ],
        ),
        (
            |count| CrudText::DeleteManyFailed { count },
            ["0 errors.", "1 error.", "2 errors."],
        ),
        (
            |count| CrudText::DeleteManySkipped { count },
            ["0 not attempted.", "1 not attempted.", "2 not attempted."],
        ),
        (
            |count| CrudText::DeleteManyDialogTitle { count },
            ["Delete 0 entries", "Delete 1 entry", "Delete 2 entries"],
        ),
        (
            |count| CrudText::DeleteManyDialogMessage { count },
            [
                "Are you sure? These 0 entries cannot be restored.",
                "Are you sure? This entry cannot be restored.",
                "Are you sure? These 2 entries cannot be restored.",
            ],
        ),
    ];

    #[test]
    fn english_defaults_choose_the_plural_forms_of_their_counts() {
        let counted = CrudText::catalog()
            .iter()
            .filter(|text| text.args().iter().any(|(name, _)| *name == "count"))
            .map(CrudText::id)
            .collect::<Vec<_>>();
        let tested = COUNT_TEXTS
            .iter()
            .map(|(text, _)| text(0).id())
            .collect::<Vec<_>>();
        assert_that!(tested).is_equal_to(counted);

        for (text, expected) in COUNT_TEXTS {
            let english = [0, 1, 2].map(|count| text(count).english().into_owned());

            assert_that!(english).is_equal_to(expected.map(str::to_owned));
        }
    }
}
