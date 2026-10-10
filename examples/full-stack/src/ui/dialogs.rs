//! Confirmation dialogs: CrudKit's dialog atoms with Leptonic's modal atoms in the app's markup.

use crate::ui::{icon, use_noun};
use crudkit_leptos::prelude::*;
use leptonic::atoms::prelude::{Dialog, DialogDescription, DialogTitle, ModalContent};
use leptonic::hooks::DialogRole;
use leptos::prelude::*;

/// The content of a confirmation dialog.
#[component]
fn Confirm(
    #[prop(into)] title: Signal<String>,
    #[prop(into)] message: String,
    #[prop(into)] cancel_label: String,
    #[prop(into)] confirm_label: String,
) -> impl IntoView {
    view! {
        <ModalContent classes="ui-dialog">
            <Dialog role=DialogRole::AlertDialog>
                <div class="ui-dialog-body">
                    <span class="ui-dialog-icon">{icon(icondata::LuTriangleAlert)}</span>
                    <div>
                        <DialogTitle classes="ui-dialog-title">{move || title.get()}</DialogTitle>
                        <DialogDescription classes="ui-dialog-message">{message}</DialogDescription>
                    </div>
                </div>
                <div class="ui-dialog-actions">
                    <CrudCancelButton classes="ui-button">{cancel_label}</CrudCancelButton>
                    <CrudConfirmButton classes=[
                        "ui-button",
                        "ui-button-danger",
                    ]>{confirm_label}</CrudConfirmButton>
                </div>
            </Dialog>
        </ModalContent>
    }
}

/// Dialogs for the pending deletions and navigation attempts of the surrounding instance.
#[component]
pub fn Dialogs() -> impl IntoView {
    let noun = use_noun();
    let pending_many = use_crud_delete().pending_many;
    let many = Signal::derive(move || {
        pending_many
            .read()
            .as_ref()
            .map_or(0, |entities| entities.len())
    });
    view! {
        <CrudDeleteDialog classes="ui-backdrop">
            <Confirm
                title=format!("Delete this {}?", noun.singular)
                message="The record vanishes from the registry for everyone. No spell can undo this."
                cancel_label="Cancel"
                confirm_label="Delete"
            />
        </CrudDeleteDialog>
        <CrudDeleteManyDialog classes="ui-backdrop">
            <Confirm
                title=Signal::derive(move || format!("Delete {}?", noun.count(many.get())))
                message="The selected records vanish from the registry for everyone. No spell can undo this."
                cancel_label="Cancel"
                confirm_label="Delete all"
            />
        </CrudDeleteManyDialog>
        <CrudLeaveDialog classes="ui-backdrop">
            <Confirm
                title="Discard unsaved changes?".to_owned()
                message="You edited this record without saving. Leaving now discards those edits."
                cancel_label="Keep editing"
                confirm_label="Discard"
            />
        </CrudLeaveDialog>
    }
}
