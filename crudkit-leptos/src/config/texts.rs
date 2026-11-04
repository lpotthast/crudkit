//! The texts that CrudKit shows or announces, and their translation.

use crudkit_web::http::RequestError;
use std::borrow::Cow;

/// A text that CrudKit shows or announces: the default content of an atom, an accessible name,
/// the message of a notification, or the explanation of an error.
///
/// Each text has a stable [ID](Self::id), named [arguments](Self::args), and an English default
/// ([`Self::english`]). A [`CrudTranslator`] looks texts up in the user's language. Without one,
/// CrudKit shows the English defaults.
///
/// The ID of a text is `crudkit-` followed by the name of its variant in kebab case, such as
/// `crudkit-form-save` for [`Self::FormSave`]. The arguments of a text are the fields of its
/// variant, by their names, such as `count` for [`Self::SelectionCount`]. IDs and argument names
/// are valid identifiers of Fluent messages and variables, so a Fluent resource can name them
/// directly. They change only in a breaking release.
///
/// The documentation of each variant names where the text shows, and its English default. A text
/// that takes a count chooses its plural form itself, and so does a translation. CrudKit joins
/// sentences of one message, such as those of a mass deletion's outcome, with a space.
///
/// A translator written in Rust can match the variants. A `match` without a wildcard arm makes the
/// compiler name every text that a new release adds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrudText<'a> {
    /// The status of a list or an entity that loads: "Loading…".
    StatusLoading,
    /// The status of a list without entities: "No data".
    StatusEmpty,
    /// The status of an entity that does not exist: "This entry does not exist."
    StatusNotFound,
    /// The status of a list or an entity that could not be loaded, given the translated `error`:
    /// "Data unavailable: {error}".
    StatusFailed {
        /// Why loading failed, e.g. the text of a [`RequestError`].
        error: &'a str,
    },
    /// The header of a table's actions column: "Actions".
    TableActions,
    /// The accessible name of the checkbox that selects all rows: "Select all".
    TableSelectAll,
    /// The accessible name of a row's selection checkbox: "Select".
    TableSelectRow,
    /// The number of selected entities: "{count} selected".
    SelectionCount {
        /// The number of selected entities.
        count: u64,
    },
    /// Deselects all entities: "Clear selection".
    SelectionClear,
    /// Deletes the selected entities: "Delete selection".
    SelectionDelete,
    /// Resets an instance to its configuration: "Reset".
    InstanceReset,
    /// The accessible name of a list's pagination: "Pages".
    PaginationLabel,
    /// Opens the previous page: "Previous page".
    PaginationPrevious,
    /// Opens the next page: "Next page".
    PaginationNext,
    /// The accessible name of a page's button: "Page {page}".
    PaginationPage {
        /// The number of the page, counted from one.
        page: u64,
    },
    /// The label of the page size selection: "Entries per page".
    PaginationPageSize,
    /// Opens the create view: "New".
    EntityCreate,
    /// Opens the read view of an entity: "View".
    EntityRead,
    /// Opens the edit view of an entity: "Edit".
    EntityEdit,
    /// Deletes an entity, also in the confirmation dialogs of deletions: "Delete".
    EntityDelete,
    /// Saves a form: "Save".
    FormSave,
    /// Saves a form and returns: "Save and return".
    FormSaveAndReturn,
    /// Saves a form and opens a fresh create view: "Save and new".
    FormSaveAndNew,
    /// Returns from the current view: "Back".
    FormReturn,
    /// Reloads the entity of a form to show its current state, keeping the input: "Show current
    /// state".
    FormReload,
    /// Explains that saving was refused, because someone else changed the entity since the form
    /// loaded it: "Someone else changed this entry in the meantime. Show its current state, then
    /// save again. Your input is kept."
    FormOutdated,
    /// Explains that the user's changes cannot be applied to the current version of the entity:
    /// "Your changes cannot be applied to the current version of this entry, which someone else
    /// changed. Leave the entry to discard your changes, then open it again."
    FormRebaseFailed,
    /// Explains that someone else changed fields that the user changed too, which a reload found:
    /// "Someone else also changed fields that you changed. Check them, then save again."
    FormConflicts,
    /// Explains that someone else changed a value that the user changed too: "Someone else
    /// changed this to "{value}" meanwhile."
    FormChangedMeanwhile {
        /// The value that someone else set, e.g. `blue`, or JSON for other values than strings.
        value: &'a str,
    },
    /// Explains that someone else removed a value that the user changed: "Someone else removed
    /// this meanwhile."
    FormRemovedMeanwhile,
    /// A problem or a conflict at a place of a form: "{location}: {message}".
    FormProblem {
        /// Where the problem is: the path of a member below a field, such as `0/count`, or in the
        /// form's error summary the field's label and that path, such as `Players/0/count`.
        location: &'a str,
        /// What the problem is, e.g. the message of the data source.
        message: &'a str,
    },
    /// The title of the notifications of saves: "Save".
    SaveTitle,
    /// The title of the notifications of failed saves: "Error".
    SaveErrorTitle,
    /// After an entity was created: "The entry was created."
    SaveCreated,
    /// After the data source accepted the creation of an entity, which it still sets up: "The
    /// entry was created and is being set up."
    SaveCreateAccepted,
    /// After creating an entity failed: "The entry could not be created. {error}".
    SaveCreateFailed {
        /// Why creating failed, e.g. the text of a [`RequestError`].
        error: &'a str,
    },
    /// After an entity was saved: "The changes were saved."
    SaveUpdated,
    /// After the data source accepted a change, which it still applies: "The changes were
    /// accepted and are being applied."
    SaveUpdateAccepted,
    /// After saving an entity failed: "The entry could not be saved. {error}".
    SaveUpdateFailed {
        /// Why saving failed, e.g. the text of a [`RequestError`].
        error: &'a str,
    },
    /// The title of the notifications of deletions: "Deletion".
    DeleteTitle,
    /// After entities were deleted: "1 entry deleted." or "{count} entries deleted."
    DeleteCompleted {
        /// The number of deleted entities.
        count: u64,
    },
    /// After the data source accepted the deletion of entities, which it still deletes: "1 entry
    /// is being deleted." or "{count} entries are being deleted."
    DeleteAccepted {
        /// The number of entities being deleted.
        count: u64,
    },
    /// After a deletion was refused, because someone else changed the entity since the user saw
    /// it: "Someone else changed this entry in the meantime. Check the changes and delete it
    /// again."
    DeleteOutdated,
    /// After the data source refused a deletion: "Deletion refused. Reason: {reason}".
    DeleteForbidden {
        /// The data source's reason.
        reason: &'a str,
    },
    /// After the data source rejected a deletion as invalid: "Deletion not possible. Reason:
    /// {reason}".
    DeleteRejected {
        /// The data source's reason.
        reason: &'a str,
    },
    /// After a deletion failed: "Deletion failed: {error}".
    DeleteFailed {
        /// Why the deletion failed, e.g. the text of a [`RequestError`].
        error: &'a str,
    },
    /// The first sentence after a mass deletion that deleted some of its entities: "1 entry
    /// deleted." or "{count} entries deleted."
    DeleteManyPartial {
        /// The number of deleted entities.
        count: u64,
    },
    /// The first sentence after a mass deletion that deleted nothing: "No entries deleted."
    DeleteManyNone,
    /// The sentence about the deletions of a mass deletion that the data source aborted:
    /// "{count} aborted."
    DeleteManyAborted {
        /// The number of aborted deletions.
        count: u64,
    },
    /// The sentence about the deletions of a mass deletion that validation prevented: "1
    /// validation error." or "{count} validation errors."
    DeleteManyInvalid {
        /// The number of prevented deletions.
        count: u64,
    },
    /// The sentence about the deletions of a mass deletion that failed: "1 error." or "{count}
    /// errors."
    DeleteManyFailed {
        /// The number of failed deletions.
        count: u64,
    },
    /// The sentence about the entities of a mass deletion that a failure left untried: "{count}
    /// not attempted."
    DeleteManySkipped {
        /// The number of untried entities.
        count: u64,
    },
    /// The title of the dialog confirming a deletion: "Delete entry".
    DeleteDialogTitle,
    /// The message of the dialog confirming a deletion: "Are you sure? This entry cannot be
    /// restored."
    DeleteDialogMessage,
    /// The title of the dialog confirming a mass deletion: "Delete 1 entry" or "Delete {count}
    /// entries".
    DeleteManyDialogTitle {
        /// The number of entities to delete.
        count: u64,
    },
    /// The message of the dialog confirming a mass deletion: "Are you sure? This entry cannot be
    /// restored." or "Are you sure? These {count} entries cannot be restored."
    DeleteManyDialogMessage {
        /// The number of entities to delete.
        count: u64,
    },
    /// The title of the dialog confirming to leave unsaved changes: "Unsaved changes".
    LeaveDialogTitle,
    /// The message of the dialog confirming to leave unsaved changes: "You have not saved your
    /// changes. Do you really want to leave? Unsaved changes are lost."
    LeaveDialogMessage,
    /// Confirms leaving and discarding the changes: "Leave".
    LeaveDialogConfirm,
    /// Cancels a confirmation dialog: "Cancel".
    DialogCancel,
    /// A `true` value shown as text: "Yes".
    ValueYes,
    /// A `false` value shown as text: "No".
    ValueNo,
    /// Rejects the input of a UUID field: "Enter a UUID, such as
    /// 67e55044-10b1-426f-9247-bb680e5fe0c8."
    InputInvalidUuid,
    /// Rejects the input of a date-time field: "Enter a date and time in the form
    /// YYYY-MM-DDTHH:MM or YYYY-MM-DDTHH:MM:SS."
    InputInvalidDateTime,
    /// Rejects the input of a date-time field with offset: "Enter a date and time with an offset,
    /// such as 2026-11-04T09:30:00+01:00."
    InputInvalidOffsetDateTime,
    /// Rejects the input of a duration field: "Enter a duration in the form HH:MM:SS or
    /// HH:MM:SS.cc."
    InputInvalidDuration,
    /// Rejects the input of a JSON field: "Invalid JSON at line {line}, column {column}."
    InputInvalidJson {
        /// The line of the first error, counted from one.
        line: u64,
        /// The column of the first error, counted from one.
        column: u64,
    },
    /// A configuration error: a field of a custom type has no registered renderer: "No renderer
    /// is registered for the field '{field}'."
    FieldNoRenderer {
        /// The name of the field.
        field: &'a str,
    },
    /// A configuration error: a form shows a field that its model does not have: "The field
    /// '{field}' is not part of this form."
    FieldUnknown {
        /// The name of the field.
        field: &'a str,
    },
    /// A configuration error: a view has no registered renderer: "No renderer is registered for
    /// the view '{view}'."
    ViewNotRegistered {
        /// The name of the view.
        view: &'a str,
    },
    /// A configuration error: a standard view without an entity was opened with one: "The view
    /// '{view}' does not accept an entity subject."
    ViewRejectsSubject {
        /// The name of the view.
        view: &'a str,
    },
    /// A configuration error: a standard view of an entity was opened without one: "The view
    /// '{view}' requires an entity subject."
    ViewNeedsSubject {
        /// The name of the view.
        view: &'a str,
    },
    /// A configuration error: a standard view was opened with a payload: "The view '{view}' does
    /// not accept a payload."
    ViewRejectsPayload {
        /// The name of the view.
        view: &'a str,
    },
    /// [`RequestError::BadRequest`]: "Bad request: {message}".
    ErrorBadRequest {
        /// The message of the error.
        message: &'a str,
    },
    /// [`RequestError::Unauthorized`]: "Unauthorized: {message}".
    ErrorUnauthorized {
        /// The message of the error.
        message: &'a str,
    },
    /// [`RequestError::Forbidden`]: "Permission denied: {message}".
    ErrorForbidden {
        /// The message of the error.
        message: &'a str,
    },
    /// [`RequestError::NotFound`]: "Not found: {message}".
    ErrorNotFound {
        /// The message of the error.
        message: &'a str,
    },
    /// [`RequestError::UnprocessableEntity`]: "Unprocessable entity: {message}".
    ErrorUnprocessable {
        /// The message of the error.
        message: &'a str,
    },
    /// [`RequestError::CriticalValidationErrors`]: "Validation failed: {message}".
    ErrorValidation {
        /// The message of the error.
        message: &'a str,
    },
    /// [`RequestError::Outdated`]: "Outdated: {message}".
    ErrorOutdated {
        /// The message of the error.
        message: &'a str,
    },
    /// [`RequestError::InvalidFields`]: "Invalid input: {message}".
    ErrorInvalidFields {
        /// The message of the error.
        message: &'a str,
    },
    /// [`RequestError::InternalServerError`]: "Server error: {message}".
    ErrorServer {
        /// The message of the error.
        message: &'a str,
    },
    /// [`RequestError::InvalidRequest`]: "Invalid request: {message}".
    ErrorInvalidRequest {
        /// The message of the error.
        message: &'a str,
    },
    /// [`RequestError::Request`]: "Network error: {message}".
    ErrorNetwork {
        /// The message of the error.
        message: &'a str,
    },
    /// [`RequestError::Deserialize`]: "Invalid response: {message}".
    ErrorInvalidResponse {
        /// The message of the error.
        message: &'a str,
    },
}

/// The value of an argument of a [`CrudText`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrudTextArg<'a> {
    /// A number, e.g. a count, from which a translation chooses a plural form.
    Number(u64),
    /// Text, e.g. the name of a field or a message of the data source.
    String(&'a str),
}

impl<'a> From<&'a RequestError> for CrudText<'a> {
    /// The text explaining `error` to the user.
    fn from(error: &'a RequestError) -> Self {
        match error {
            RequestError::BadRequest(message) => Self::ErrorBadRequest { message },
            RequestError::Unauthorized(message) => Self::ErrorUnauthorized { message },
            RequestError::Forbidden(message) => Self::ErrorForbidden { message },
            RequestError::NotFound(message) => Self::ErrorNotFound { message },
            RequestError::UnprocessableEntity(message) => Self::ErrorUnprocessable { message },
            RequestError::CriticalValidationErrors { message, .. } => {
                Self::ErrorValidation { message }
            }
            RequestError::Outdated(message) => Self::ErrorOutdated { message },
            RequestError::InvalidFields { message, .. } => Self::ErrorInvalidFields { message },
            RequestError::InternalServerError(message) => Self::ErrorServer { message },
            RequestError::InvalidRequest(message) => Self::ErrorInvalidRequest { message },
            RequestError::Request(message) => Self::ErrorNetwork { message },
            RequestError::Deserialize(message) => Self::ErrorInvalidResponse { message },
        }
    }
}

impl<'a> CrudText<'a> {
    /// Returns the ID of this text: `crudkit-` followed by the name of its variant in kebab case.
    #[must_use]
    // One line per text, so the function grows with the texts.
    #[allow(clippy::too_many_lines)]
    pub fn id(&self) -> &'static str {
        match self {
            Self::StatusLoading => "crudkit-status-loading",
            Self::StatusEmpty => "crudkit-status-empty",
            Self::StatusNotFound => "crudkit-status-not-found",
            Self::StatusFailed { .. } => "crudkit-status-failed",
            Self::TableActions => "crudkit-table-actions",
            Self::TableSelectAll => "crudkit-table-select-all",
            Self::TableSelectRow => "crudkit-table-select-row",
            Self::SelectionCount { .. } => "crudkit-selection-count",
            Self::SelectionClear => "crudkit-selection-clear",
            Self::SelectionDelete => "crudkit-selection-delete",
            Self::InstanceReset => "crudkit-instance-reset",
            Self::PaginationLabel => "crudkit-pagination-label",
            Self::PaginationPrevious => "crudkit-pagination-previous",
            Self::PaginationNext => "crudkit-pagination-next",
            Self::PaginationPage { .. } => "crudkit-pagination-page",
            Self::PaginationPageSize => "crudkit-pagination-page-size",
            Self::EntityCreate => "crudkit-entity-create",
            Self::EntityRead => "crudkit-entity-read",
            Self::EntityEdit => "crudkit-entity-edit",
            Self::EntityDelete => "crudkit-entity-delete",
            Self::FormSave => "crudkit-form-save",
            Self::FormSaveAndReturn => "crudkit-form-save-and-return",
            Self::FormSaveAndNew => "crudkit-form-save-and-new",
            Self::FormReturn => "crudkit-form-return",
            Self::FormReload => "crudkit-form-reload",
            Self::FormOutdated => "crudkit-form-outdated",
            Self::FormRebaseFailed => "crudkit-form-rebase-failed",
            Self::FormConflicts => "crudkit-form-conflicts",
            Self::FormChangedMeanwhile { .. } => "crudkit-form-changed-meanwhile",
            Self::FormRemovedMeanwhile => "crudkit-form-removed-meanwhile",
            Self::FormProblem { .. } => "crudkit-form-problem",
            Self::SaveTitle => "crudkit-save-title",
            Self::SaveErrorTitle => "crudkit-save-error-title",
            Self::SaveCreated => "crudkit-save-created",
            Self::SaveCreateAccepted => "crudkit-save-create-accepted",
            Self::SaveCreateFailed { .. } => "crudkit-save-create-failed",
            Self::SaveUpdated => "crudkit-save-updated",
            Self::SaveUpdateAccepted => "crudkit-save-update-accepted",
            Self::SaveUpdateFailed { .. } => "crudkit-save-update-failed",
            Self::DeleteTitle => "crudkit-delete-title",
            Self::DeleteCompleted { .. } => "crudkit-delete-completed",
            Self::DeleteAccepted { .. } => "crudkit-delete-accepted",
            Self::DeleteOutdated => "crudkit-delete-outdated",
            Self::DeleteForbidden { .. } => "crudkit-delete-forbidden",
            Self::DeleteRejected { .. } => "crudkit-delete-rejected",
            Self::DeleteFailed { .. } => "crudkit-delete-failed",
            Self::DeleteManyPartial { .. } => "crudkit-delete-many-partial",
            Self::DeleteManyNone => "crudkit-delete-many-none",
            Self::DeleteManyAborted { .. } => "crudkit-delete-many-aborted",
            Self::DeleteManyInvalid { .. } => "crudkit-delete-many-invalid",
            Self::DeleteManyFailed { .. } => "crudkit-delete-many-failed",
            Self::DeleteManySkipped { .. } => "crudkit-delete-many-skipped",
            Self::DeleteDialogTitle => "crudkit-delete-dialog-title",
            Self::DeleteDialogMessage => "crudkit-delete-dialog-message",
            Self::DeleteManyDialogTitle { .. } => "crudkit-delete-many-dialog-title",
            Self::DeleteManyDialogMessage { .. } => "crudkit-delete-many-dialog-message",
            Self::LeaveDialogTitle => "crudkit-leave-dialog-title",
            Self::LeaveDialogMessage => "crudkit-leave-dialog-message",
            Self::LeaveDialogConfirm => "crudkit-leave-dialog-confirm",
            Self::DialogCancel => "crudkit-dialog-cancel",
            Self::ValueYes => "crudkit-value-yes",
            Self::ValueNo => "crudkit-value-no",
            Self::InputInvalidUuid => "crudkit-input-invalid-uuid",
            Self::InputInvalidDateTime => "crudkit-input-invalid-date-time",
            Self::InputInvalidOffsetDateTime => "crudkit-input-invalid-offset-date-time",
            Self::InputInvalidDuration => "crudkit-input-invalid-duration",
            Self::InputInvalidJson { .. } => "crudkit-input-invalid-json",
            Self::FieldNoRenderer { .. } => "crudkit-field-no-renderer",
            Self::FieldUnknown { .. } => "crudkit-field-unknown",
            Self::ViewNotRegistered { .. } => "crudkit-view-not-registered",
            Self::ViewRejectsSubject { .. } => "crudkit-view-rejects-subject",
            Self::ViewNeedsSubject { .. } => "crudkit-view-needs-subject",
            Self::ViewRejectsPayload { .. } => "crudkit-view-rejects-payload",
            Self::ErrorBadRequest { .. } => "crudkit-error-bad-request",
            Self::ErrorUnauthorized { .. } => "crudkit-error-unauthorized",
            Self::ErrorForbidden { .. } => "crudkit-error-forbidden",
            Self::ErrorNotFound { .. } => "crudkit-error-not-found",
            Self::ErrorUnprocessable { .. } => "crudkit-error-unprocessable",
            Self::ErrorValidation { .. } => "crudkit-error-validation",
            Self::ErrorOutdated { .. } => "crudkit-error-outdated",
            Self::ErrorInvalidFields { .. } => "crudkit-error-invalid-fields",
            Self::ErrorServer { .. } => "crudkit-error-server",
            Self::ErrorInvalidRequest { .. } => "crudkit-error-invalid-request",
            Self::ErrorNetwork { .. } => "crudkit-error-network",
            Self::ErrorInvalidResponse { .. } => "crudkit-error-invalid-response",
        }
    }

    /// Returns the arguments of this text: the fields of its variant, by their names, in the order
    /// of the fields. Empty for a text without arguments.
    #[must_use]
    pub fn args(&self) -> Vec<(&'static str, CrudTextArg<'a>)> {
        use CrudTextArg::{Number, String as Text};
        match *self {
            Self::StatusLoading
            | Self::StatusEmpty
            | Self::StatusNotFound
            | Self::TableActions
            | Self::TableSelectAll
            | Self::TableSelectRow
            | Self::SelectionClear
            | Self::SelectionDelete
            | Self::InstanceReset
            | Self::PaginationLabel
            | Self::PaginationPrevious
            | Self::PaginationNext
            | Self::PaginationPageSize
            | Self::EntityCreate
            | Self::EntityRead
            | Self::EntityEdit
            | Self::EntityDelete
            | Self::FormSave
            | Self::FormSaveAndReturn
            | Self::FormSaveAndNew
            | Self::FormReturn
            | Self::FormReload
            | Self::FormOutdated
            | Self::FormRebaseFailed
            | Self::FormConflicts
            | Self::FormRemovedMeanwhile
            | Self::SaveTitle
            | Self::SaveErrorTitle
            | Self::SaveCreated
            | Self::SaveCreateAccepted
            | Self::SaveUpdated
            | Self::SaveUpdateAccepted
            | Self::DeleteTitle
            | Self::DeleteOutdated
            | Self::DeleteManyNone
            | Self::DeleteDialogTitle
            | Self::DeleteDialogMessage
            | Self::LeaveDialogTitle
            | Self::LeaveDialogMessage
            | Self::LeaveDialogConfirm
            | Self::DialogCancel
            | Self::ValueYes
            | Self::ValueNo
            | Self::InputInvalidUuid
            | Self::InputInvalidDateTime
            | Self::InputInvalidOffsetDateTime
            | Self::InputInvalidDuration => Vec::new(),
            Self::SelectionCount { count }
            | Self::DeleteCompleted { count }
            | Self::DeleteAccepted { count }
            | Self::DeleteManyPartial { count }
            | Self::DeleteManyAborted { count }
            | Self::DeleteManyInvalid { count }
            | Self::DeleteManyFailed { count }
            | Self::DeleteManySkipped { count }
            | Self::DeleteManyDialogTitle { count }
            | Self::DeleteManyDialogMessage { count } => vec![("count", Number(count))],
            Self::PaginationPage { page } => vec![("page", Number(page))],
            Self::InputInvalidJson { line, column } => {
                vec![("line", Number(line)), ("column", Number(column))]
            }
            Self::StatusFailed { error }
            | Self::SaveCreateFailed { error }
            | Self::SaveUpdateFailed { error }
            | Self::DeleteFailed { error } => vec![("error", Text(error))],
            Self::FormChangedMeanwhile { value } => vec![("value", Text(value))],
            Self::FormProblem { location, message } => {
                vec![("location", Text(location)), ("message", Text(message))]
            }
            Self::DeleteForbidden { reason } | Self::DeleteRejected { reason } => {
                vec![("reason", Text(reason))]
            }
            Self::FieldNoRenderer { field } | Self::FieldUnknown { field } => {
                vec![("field", Text(field))]
            }
            Self::ViewNotRegistered { view }
            | Self::ViewRejectsSubject { view }
            | Self::ViewNeedsSubject { view }
            | Self::ViewRejectsPayload { view } => vec![("view", Text(view))],
            Self::ErrorBadRequest { message }
            | Self::ErrorUnauthorized { message }
            | Self::ErrorForbidden { message }
            | Self::ErrorNotFound { message }
            | Self::ErrorUnprocessable { message }
            | Self::ErrorValidation { message }
            | Self::ErrorOutdated { message }
            | Self::ErrorInvalidFields { message }
            | Self::ErrorServer { message }
            | Self::ErrorInvalidRequest { message }
            | Self::ErrorNetwork { message }
            | Self::ErrorInvalidResponse { message } => vec![("message", Text(message))],
        }
    }

    /// Returns the English default of this text, with its arguments.
    #[must_use]
    // One line or a few per text, so the function grows with the texts.
    #[allow(clippy::too_many_lines)]
    pub fn english(&self) -> Cow<'static, str> {
        let entries = |count: u64| match count {
            1 => "1 entry".to_owned(),
            _ => format!("{count} entries"),
        };
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
            Self::FormSave | Self::SaveTitle => "Save".into(),
            Self::FormSaveAndReturn => "Save and return".into(),
            Self::FormSaveAndNew => "Save and new".into(),
            Self::FormReturn => "Back".into(),
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
                format!("Someone else changed this to \"{value}\" meanwhile.").into()
            }
            Self::FormRemovedMeanwhile => "Someone else removed this meanwhile.".into(),
            Self::FormProblem { location, message } => format!("{location}: {message}").into(),
            Self::SaveErrorTitle => "Error".into(),
            Self::SaveCreated => "The entry was created.".into(),
            Self::SaveCreateAccepted => "The entry was created and is being set up.".into(),
            Self::SaveCreateFailed { error } => {
                format!("The entry could not be created. {error}").into()
            }
            Self::SaveUpdated => "The changes were saved.".into(),
            Self::SaveUpdateAccepted => "The changes were accepted and are being applied.".into(),
            Self::SaveUpdateFailed { error } => {
                format!("The entry could not be saved. {error}").into()
            }
            Self::DeleteTitle => "Deletion".into(),
            Self::DeleteCompleted { count } | Self::DeleteManyPartial { count } => {
                format!("{} deleted.", entries(count)).into()
            }
            Self::DeleteAccepted { count } => match count {
                1 => "1 entry is being deleted.".into(),
                _ => format!("{count} entries are being deleted.").into(),
            },
            Self::DeleteOutdated => "Someone else changed this entry in the meantime. Check the \
                                     changes and delete it again."
                .into(),
            Self::DeleteForbidden { reason } => {
                format!("Deletion refused. Reason: {reason}").into()
            }
            Self::DeleteRejected { reason } => {
                format!("Deletion not possible. Reason: {reason}").into()
            }
            Self::DeleteFailed { error } => format!("Deletion failed: {error}").into(),
            Self::DeleteManyNone => "No entries deleted.".into(),
            Self::DeleteManyAborted { count } => format!("{count} aborted.").into(),
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
                "Enter a duration in the form HH:MM:SS or HH:MM:SS.cc.".into()
            }
            Self::InputInvalidJson { line, column } => {
                format!("Invalid JSON at line {line}, column {column}.").into()
            }
            Self::FieldNoRenderer { field } => {
                format!("No renderer is registered for the field '{field}'.").into()
            }
            Self::FieldUnknown { field } => {
                format!("The field '{field}' is not part of this form.").into()
            }
            Self::ViewNotRegistered { view } => {
                format!("No renderer is registered for the view '{view}'.").into()
            }
            Self::ViewRejectsSubject { view } => {
                format!("The view '{view}' does not accept an entity subject.").into()
            }
            Self::ViewNeedsSubject { view } => {
                format!("The view '{view}' requires an entity subject.").into()
            }
            Self::ViewRejectsPayload { view } => {
                format!("The view '{view}' does not accept a payload.").into()
            }
            Self::ErrorBadRequest { message } => format!("Bad request: {message}").into(),
            Self::ErrorUnauthorized { message } => format!("Unauthorized: {message}").into(),
            Self::ErrorForbidden { message } => format!("Permission denied: {message}").into(),
            Self::ErrorNotFound { message } => format!("Not found: {message}").into(),
            Self::ErrorUnprocessable { message } => {
                format!("Unprocessable entity: {message}").into()
            }
            Self::ErrorValidation { message } => format!("Validation failed: {message}").into(),
            Self::ErrorOutdated { message } => format!("Outdated: {message}").into(),
            Self::ErrorInvalidFields { message } => format!("Invalid input: {message}").into(),
            Self::ErrorServer { message } => format!("Server error: {message}").into(),
            Self::ErrorInvalidRequest { message } => format!("Invalid request: {message}").into(),
            Self::ErrorNetwork { message } => format!("Network error: {message}").into(),
            Self::ErrorInvalidResponse { message } => format!("Invalid response: {message}").into(),
        }
    }
}

/// Looks CrudKit's texts up in the user's language.
///
/// Provide a translator once, at the root of the application, with
/// [`provide_crud_translator`](crate::hooks::provide_crud_translator). Without one, CrudKit shows
/// the English defaults ([`CrudText::english`]). A translator may translate some texts only: CrudKit
/// shows the English default of every text that the translator returns `None` for. A closure
/// `Fn(&CrudText<'_>) -> Option<String>` is a translator too.
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
pub(crate) mod tests {
    use super::*;
    use assertr::prelude::*;
    use std::collections::HashSet;

    /// One text of every variant, with arguments. Add every new text here.
    // One line per text, so the function grows with the texts.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn every_text() -> Vec<CrudText<'static>> {
        vec![
            CrudText::StatusLoading,
            CrudText::StatusEmpty,
            CrudText::StatusNotFound,
            CrudText::StatusFailed { error: "offline" },
            CrudText::TableActions,
            CrudText::TableSelectAll,
            CrudText::TableSelectRow,
            CrudText::SelectionCount { count: 3 },
            CrudText::SelectionClear,
            CrudText::SelectionDelete,
            CrudText::InstanceReset,
            CrudText::PaginationLabel,
            CrudText::PaginationPrevious,
            CrudText::PaginationNext,
            CrudText::PaginationPage { page: 2 },
            CrudText::PaginationPageSize,
            CrudText::EntityCreate,
            CrudText::EntityRead,
            CrudText::EntityEdit,
            CrudText::EntityDelete,
            CrudText::FormSave,
            CrudText::FormSaveAndReturn,
            CrudText::FormSaveAndNew,
            CrudText::FormReturn,
            CrudText::FormReload,
            CrudText::FormOutdated,
            CrudText::FormRebaseFailed,
            CrudText::FormConflicts,
            CrudText::FormChangedMeanwhile { value: "blue" },
            CrudText::FormRemovedMeanwhile,
            CrudText::FormProblem {
                location: "0/count",
                message: "must be positive",
            },
            CrudText::SaveTitle,
            CrudText::SaveErrorTitle,
            CrudText::SaveCreated,
            CrudText::SaveCreateAccepted,
            CrudText::SaveCreateFailed { error: "offline" },
            CrudText::SaveUpdated,
            CrudText::SaveUpdateAccepted,
            CrudText::SaveUpdateFailed { error: "offline" },
            CrudText::DeleteTitle,
            CrudText::DeleteCompleted { count: 3 },
            CrudText::DeleteAccepted { count: 3 },
            CrudText::DeleteOutdated,
            CrudText::DeleteForbidden { reason: "locked" },
            CrudText::DeleteRejected { reason: "in use" },
            CrudText::DeleteFailed { error: "offline" },
            CrudText::DeleteManyPartial { count: 3 },
            CrudText::DeleteManyNone,
            CrudText::DeleteManyAborted { count: 3 },
            CrudText::DeleteManyInvalid { count: 3 },
            CrudText::DeleteManyFailed { count: 3 },
            CrudText::DeleteManySkipped { count: 3 },
            CrudText::DeleteDialogTitle,
            CrudText::DeleteDialogMessage,
            CrudText::DeleteManyDialogTitle { count: 3 },
            CrudText::DeleteManyDialogMessage { count: 3 },
            CrudText::LeaveDialogTitle,
            CrudText::LeaveDialogMessage,
            CrudText::LeaveDialogConfirm,
            CrudText::DialogCancel,
            CrudText::ValueYes,
            CrudText::ValueNo,
            CrudText::InputInvalidUuid,
            CrudText::InputInvalidDateTime,
            CrudText::InputInvalidOffsetDateTime,
            CrudText::InputInvalidDuration,
            CrudText::InputInvalidJson { line: 2, column: 7 },
            CrudText::FieldNoRenderer { field: "tags" },
            CrudText::FieldUnknown { field: "tags" },
            CrudText::ViewNotRegistered { view: "app.audit" },
            CrudText::ViewRejectsSubject { view: "table" },
            CrudText::ViewNeedsSubject { view: "edit" },
            CrudText::ViewRejectsPayload { view: "table" },
            CrudText::ErrorBadRequest { message: "m" },
            CrudText::ErrorUnauthorized { message: "m" },
            CrudText::ErrorForbidden { message: "m" },
            CrudText::ErrorNotFound { message: "m" },
            CrudText::ErrorUnprocessable { message: "m" },
            CrudText::ErrorValidation { message: "m" },
            CrudText::ErrorOutdated { message: "m" },
            CrudText::ErrorInvalidFields { message: "m" },
            CrudText::ErrorServer { message: "m" },
            CrudText::ErrorInvalidRequest { message: "m" },
            CrudText::ErrorNetwork { message: "m" },
            CrudText::ErrorInvalidResponse { message: "m" },
        ]
    }

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

    /// Whether `name` is an identifier of Fluent: `[a-zA-Z][a-zA-Z0-9_-]*`.
    fn is_fluent_identifier(name: &str) -> bool {
        let mut characters = name.chars();
        characters
            .next()
            .is_some_and(|first| first.is_ascii_alphabetic())
            && characters.all(|it| it.is_ascii_alphanumeric() || it == '_' || it == '-')
    }

    #[test]
    fn ids_name_their_variant_in_kebab_case_and_are_unique() {
        let texts = every_text();

        let ids = texts.iter().map(CrudText::id).collect::<Vec<_>>();

        for (text, id) in texts.iter().zip(&ids) {
            let expected = format!("crudkit-{}", kebab_case(&variant_name(text)));
            assert_that!(*id).is_equal_to(expected.as_str());
            assert_that!(is_fluent_identifier(id)).is_true();
        }
        assert_that!(ids.iter().collect::<HashSet<_>>().len()).is_equal_to(ids.len());
    }

    /// Returns the names of the fields of the variant of `text`, read from its `Debug` output. The
    /// values of [`every_text`] contain no `{`, `,`, or `:`, so that they do not end a name.
    fn field_names(text: &CrudText<'_>) -> Vec<String> {
        format!("{text:?}")
            .split(['{', ','])
            .skip(1)
            .filter_map(|field| field.split(':').next())
            .map(|name| name.trim().to_owned())
            .collect()
    }

    #[test]
    fn arguments_are_the_fields_of_their_variant() {
        for text in every_text() {
            let names = text
                .args()
                .into_iter()
                .map(|(name, _)| name.to_owned())
                .collect::<Vec<_>>();

            assert_that!(names.iter().all(|name| is_fluent_identifier(name))).is_true();
            assert_that!(names).is_equal_to(field_names(&text));
        }
    }

    #[test]
    fn english_defaults_choose_their_plural_forms() {
        let one = CrudText::DeleteCompleted { count: 1 }.english();
        let many = CrudText::DeleteCompleted { count: 3 }.english();
        let dialog_one = CrudText::DeleteManyDialogMessage { count: 1 }.english();
        let dialog_many = CrudText::DeleteManyDialogMessage { count: 3 }.english();

        assert_that!(one.as_ref()).is_equal_to("1 entry deleted.");
        assert_that!(many.as_ref()).is_equal_to("3 entries deleted.");
        assert_that!(dialog_one.as_ref())
            .is_equal_to(CrudText::DeleteDialogMessage.english().as_ref());
        assert_that!(dialog_many.as_ref())
            .is_equal_to("Are you sure? These 3 entries cannot be restored.");
    }

    #[test]
    fn english_defaults_contain_their_arguments() {
        for text in every_text() {
            let english = text.english();

            for (_, value) in text.args() {
                let value = match value {
                    CrudTextArg::Number(number) => number.to_string(),
                    CrudTextArg::String(string) => string.to_owned(),
                };
                assert_that!(english.as_ref()).contains(value.as_str());
            }
        }
    }

    #[test]
    fn request_errors_are_explained_by_their_kind_and_message() {
        let error = RequestError::Forbidden("not yours".to_owned());

        let text = CrudText::from(&error);

        assert_that!(text).is_equal_to(CrudText::ErrorForbidden {
            message: "not yours",
        });
        assert_that!(text.english().as_ref()).is_equal_to("Permission denied: not yours");
    }
}
