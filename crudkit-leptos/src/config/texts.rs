//! User-facing texts shown or announced by CrudKit's atoms and hooks.

use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

type Text = Cow<'static, str>;
type CountText = Arc<dyn Fn(u64) -> String + Send + Sync>;
type ReasonText = Arc<dyn Fn(&str) -> String + Send + Sync>;

/// Texts shown or announced by CrudKit: the default content of atoms, notifications, and
/// accessible names. Applications use the dialog texts (e.g. [`Self::delete_title`]) when composing
/// confirmation dialogs.
///
/// Provide texts with [`crate::hooks::texts::provide_crud_texts`], e.g. [`CrudUiTexts::english`].
/// The defaults are [`CrudUiTexts::german`].
// TODO: Replace with proper localization.
#[derive(Clone)]
pub struct CrudUiTexts {
    /// Creates a new entity.
    pub new: Text,
    /// A field of a custom type has no registered renderer, given the field's name.
    pub no_renderer: ReasonText,
    /// A form shows a field its model does not have, given the field's name.
    pub unknown_field: ReasonText,
    /// A list or entity is loading.
    pub loading: Text,
    /// A `true` value shown as text.
    pub yes: Text,
    /// A `false` value shown as text.
    pub no: Text,
    /// Header of the row action column.
    pub actions: Text,
    /// Shown for a list without entities.
    pub no_data: Text,
    /// Shown when an entity or list could not be loaded.
    pub data_unavailable: Text,
    /// Shown when a requested entity does not exist.
    pub not_found: Text,
    /// Label of the page size selection.
    pub items_per_page: Text,
    /// Accessible name of the pagination navigation.
    pub pagination: Text,
    /// Previous page.
    pub previous_page: Text,
    /// Next page.
    pub next_page: Text,
    /// Accessible name of a page button, given the page number.
    pub page: CountText,
    /// Accessible name of the select-all checkbox.
    pub select_all: Text,
    /// Accessible name of a row selection checkbox.
    pub select_row: Text,
    /// Number of selected entities.
    pub selected: CountText,
    /// Deselects all entities.
    pub clear_selection: Text,
    /// Resets an instance to its configuration.
    pub reset: Text,
    /// Deletes the selected entities.
    pub delete_selection: Text,
    /// Opens the read view of a row.
    pub view: Text,
    /// Opens the edit view of a row.
    pub edit: Text,
    /// Saves the draft.
    pub save: Text,
    /// Saves the draft and returns.
    pub save_and_back: Text,
    /// Saves the draft and opens a fresh create view.
    pub save_and_new: Text,
    /// Deletes an entity.
    pub delete: Text,
    /// Returns from the current view.
    pub back: Text,
    /// Cancels a dialog.
    pub cancel: Text,
    /// Title of the single deletion dialog.
    pub delete_title: Text,
    /// Body of the single deletion dialog.
    pub delete_body: Text,
    /// Title and confirm label of the mass deletion dialog, given the number of entities.
    pub delete_many_title: CountText,
    /// Body of the mass deletion dialog, given the number of entities.
    pub delete_many_body: CountText,
    /// Title of the leave confirmation dialog.
    pub leave_title: Text,
    /// Body of the leave confirmation dialog.
    pub leave_body: Text,
    /// Confirms leaving and discarding changes.
    pub leave: Text,
    /// Title of error notifications.
    pub error: Text,
    /// Message after an entity was created.
    pub created: Text,
    /// Message after an entity was saved.
    pub saved: Text,
    /// Message after creating an entity failed.
    pub create_failed: Text,
    /// Message after saving an entity failed.
    pub update_failed: Text,
    /// Title of deletion notifications.
    pub deletion: Text,
    /// Message after entities were deleted, given their number.
    pub deleted: CountText,
    /// Message after some, but not all, entities were deleted, given the number deleted.
    pub deleted_partially: CountText,
    /// Message after a mass deletion deleted nothing.
    pub nothing_deleted: Text,
    /// Sentence reporting deletions aborted by the server, given their number.
    pub deletions_aborted: CountText,
    /// Sentence reporting deletions prevented by validation, given their number.
    pub deletions_invalid: CountText,
    /// Sentence reporting failed deletions, given their number.
    pub deletions_failed: CountText,
    /// Message after a deletion was refused, given the server's reason.
    pub deletion_forbidden: ReasonText,
    /// Message after a deletion was rejected as invalid, given the server's reason.
    pub deletion_rejected: ReasonText,
    /// Message after a deletion failed, given the error.
    pub deletion_failed: ReasonText,
}

impl fmt::Debug for CrudUiTexts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrudUiTexts").finish_non_exhaustive()
    }
}

impl Default for CrudUiTexts {
    /// The German texts, see [`CrudUiTexts::german`].
    fn default() -> Self {
        Self::german()
    }
}

impl CrudUiTexts {
    /// German texts.
    #[must_use]
    pub fn german() -> Self {
        let entries = |count: u64| if count == 1 { "Eintrag" } else { "Einträge" };
        Self {
            new: "Neu".into(),
            no_renderer: Arc::new(|field| {
                format!("Für das Feld '{field}' ist kein Renderer registriert.")
            }),
            unknown_field: Arc::new(|field| {
                format!("Das Feld '{field}' gehört nicht zu diesem Formular.")
            }),
            loading: "Lädt…".into(),
            yes: "Ja".into(),
            no: "Nein".into(),
            actions: "Aktionen".into(),
            no_data: "Keine Daten".into(),
            data_unavailable: "Daten nicht verfügbar".into(),
            not_found: "Eintrag existiert nicht.".into(),
            items_per_page: "Einträge pro Seite".into(),
            pagination: "Seiten".into(),
            previous_page: "Vorherige Seite".into(),
            next_page: "Nächste Seite".into(),
            page: Arc::new(|page| format!("Seite {page}")),
            select_all: "Alle auswählen".into(),
            select_row: "Auswählen".into(),
            selected: Arc::new(|count| format!("{count} ausgewählt")),
            clear_selection: "Auswahl aufheben".into(),
            reset: "Zurücksetzen".into(),
            delete_selection: "Auswahl löschen".into(),
            view: "Ansehen".into(),
            edit: "Bearbeiten".into(),
            save: "Speichern".into(),
            save_and_back: "Speichern und zurück".into(),
            save_and_new: "Speichern und neu".into(),
            delete: "Löschen".into(),
            back: "Zurück".into(),
            cancel: "Abbrechen".into(),
            delete_title: "Eintrag löschen".into(),
            delete_body: "Bist du dir sicher? Dieser Eintrag kann nicht wiederhergestellt werden!"
                .into(),
            delete_many_title: Arc::new(move |count| format!("{count} {} löschen", entries(count))),
            delete_many_body: Arc::new(move |count| match count {
                1 => "Bist du dir sicher? Dieser Eintrag kann nicht wiederhergestellt werden!"
                    .to_owned(),
                _ => format!(
                    "Bist du dir sicher? Diese {count} Einträge können nicht wiederhergestellt werden!"
                ),
            }),
            leave_title: "Ungespeicherte Änderungen".into(),
            leave_body:
                "Du hast deine Änderungen noch nicht gespeichert. Möchtest du den Bereich wirklich \
                         verlassen? Ungespeicherte Änderungen gehen verloren!"
                    .into(),
            leave: "Verlassen".into(),
            error: "Fehler".into(),
            created: "Eintrag wurde erstellt.".into(),
            saved: "Änderungen wurden gespeichert.".into(),
            create_failed: "Eintrag konnte nicht erstellt werden.".into(),
            update_failed: "Eintrag konnte nicht gespeichert werden.".into(),
            deletion: "Löschen".into(),
            deleted: Arc::new(move |count| {
                format!("{count} {} erfolgreich gelöscht.", entries(count))
            }),
            deleted_partially: Arc::new(move |count| {
                format!("{count} {} gelöscht.", entries(count))
            }),
            nothing_deleted: "Keine Einträge gelöscht.".into(),
            deletions_aborted: Arc::new(|count| format!("{count} abgebrochen.")),
            deletions_invalid: Arc::new(|count| format!("{count} Validierungsfehler.")),
            deletions_failed: Arc::new(|count| format!("{count} Fehler.")),
            deletion_forbidden: Arc::new(|reason| {
                format!("Löschvorgang abgebrochen. Grund: {reason}")
            }),
            deletion_rejected: Arc::new(|reason| {
                format!("Löschvorgang nicht möglich. Grund: {reason}")
            }),
            deletion_failed: Arc::new(|error| format!("Löschen fehlgeschlagen: {error}")),
        }
    }
    /// English texts.
    #[must_use]
    pub fn english() -> Self {
        let entries = |count: u64| if count == 1 { "entry" } else { "entries" };
        Self {
            new: "New".into(),
            no_renderer: Arc::new(|field| {
                format!("No renderer is registered for the field '{field}'.")
            }),
            unknown_field: Arc::new(|field| {
                format!("The field '{field}' is not part of this form.")
            }),
            loading: "Loading…".into(),
            yes: "Yes".into(),
            no: "No".into(),
            actions: "Actions".into(),
            no_data: "No data".into(),
            data_unavailable: "Data unavailable".into(),
            not_found: "This entry does not exist.".into(),
            items_per_page: "Entries per page".into(),
            pagination: "Pages".into(),
            previous_page: "Previous page".into(),
            next_page: "Next page".into(),
            page: Arc::new(|page| format!("Page {page}")),
            select_all: "Select all".into(),
            select_row: "Select".into(),
            selected: Arc::new(|count| format!("{count} selected")),
            clear_selection: "Clear selection".into(),
            reset: "Reset".into(),
            delete_selection: "Delete selection".into(),
            view: "View".into(),
            edit: "Edit".into(),
            save: "Save".into(),
            save_and_back: "Save and return".into(),
            save_and_new: "Save and new".into(),
            delete: "Delete".into(),
            back: "Back".into(),
            cancel: "Cancel".into(),
            delete_title: "Delete entry".into(),
            delete_body: "Are you sure? This entry cannot be restored.".into(),
            delete_many_title: Arc::new(move |count| format!("Delete {count} {}", entries(count))),
            delete_many_body: Arc::new(move |count| match count {
                1 => "Are you sure? This entry cannot be restored.".to_owned(),
                _ => format!("Are you sure? These {count} entries cannot be restored."),
            }),
            leave_title: "Unsaved changes".into(),
            leave_body: "You have not saved your changes. Do you really want to leave? Unsaved \
                         changes are lost."
                .into(),
            leave: "Leave".into(),
            error: "Error".into(),
            created: "The entry was created.".into(),
            saved: "The changes were saved.".into(),
            create_failed: "The entry could not be created.".into(),
            update_failed: "The entry could not be saved.".into(),
            deletion: "Deletion".into(),
            deleted: Arc::new(move |count| format!("{count} {} deleted.", entries(count))),
            deleted_partially: Arc::new(move |count| {
                format!("{count} {} deleted.", entries(count))
            }),
            nothing_deleted: "No entries deleted.".into(),
            deletions_aborted: Arc::new(|count| format!("{count} aborted.")),
            deletions_invalid: Arc::new(|count| format!("{count} validation errors.")),
            deletions_failed: Arc::new(|count| format!("{count} errors.")),
            deletion_forbidden: Arc::new(|reason| format!("Deletion refused. Reason: {reason}")),
            deletion_rejected: Arc::new(|reason| {
                format!("Deletion not possible. Reason: {reason}")
            }),
            deletion_failed: Arc::new(|error| format!("Deletion failed: {error}")),
        }
    }
}
