//! User-facing texts of CrudKit's built-in UI.

use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

type Text = Cow<'static, str>;
type CountText = Arc<dyn Fn(u64) -> String + Send + Sync>;
type ReasonText = Arc<dyn Fn(&str) -> String + Send + Sync>;

/// Texts rendered or announced by CrudKit.
///
/// Provide adjusted texts with [`crate::hooks::texts::provide_crud_texts`]. The defaults are
/// German.
// TODO: Replace with proper localization.
#[derive(Clone)]
pub struct CrudUiTexts {
    /// Creates a new entity.
    pub new: Text,
    /// Resets list paging and ordering.
    pub reset: Text,
    /// Header of the row action column.
    pub actions: Text,
    /// Shown for a list without entities.
    pub no_data: Text,
    /// Shown when an entity or list could not be loaded.
    pub data_unavailable: Text,
    /// Shown when a requested entity does not exist.
    pub not_found: Text,
    /// Shown for a create view without configured fields.
    pub no_fields: Text,
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
    /// Dismisses a notification.
    pub dismiss: Text,
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
    /// Message after creating an entity failed.
    pub create_failed: Text,
    /// Message after saving an entity failed.
    pub update_failed: Text,
    /// Marks an optional field as having a value.
    pub has_value: Text,
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
    fn default() -> Self {
        let entries = |count: u64| if count == 1 { "Eintrag" } else { "Einträge" };
        Self {
            new: "Neu".into(),
            reset: "Zurücksetzen".into(),
            actions: "Aktionen".into(),
            no_data: "Keine Daten".into(),
            data_unavailable: "Daten nicht verfügbar".into(),
            not_found: "Eintrag existiert nicht.".into(),
            no_fields: "Keine Felder definiert.".into(),
            items_per_page: "Einträge pro Seite".into(),
            pagination: "Seiten".into(),
            previous_page: "Vorherige Seite".into(),
            next_page: "Nächste Seite".into(),
            page: Arc::new(|page| format!("Seite {page}")),
            select_all: "Alle auswählen".into(),
            select_row: "Auswählen".into(),
            selected: Arc::new(|count| format!("{count} ausgewählt")),
            delete_selection: "Auswahl löschen".into(),
            view: "Ansehen".into(),
            edit: "Bearbeiten".into(),
            save: "Speichern".into(),
            save_and_back: "Speichern und zurück".into(),
            save_and_new: "Speichern und neu".into(),
            delete: "Löschen".into(),
            back: "Zurück".into(),
            cancel: "Abbrechen".into(),
            dismiss: "Schließen".into(),
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
            create_failed: "Eintrag konnte nicht erstellt werden.".into(),
            update_failed: "Eintrag konnte nicht gespeichert werden.".into(),
            has_value: "Wert angeben".into(),
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
}
