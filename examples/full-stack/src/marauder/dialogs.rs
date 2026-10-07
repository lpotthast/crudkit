//! Confirmation dialogs on Leptonic's modal atoms, driven by CrudKit's pending requests.

use crate::marauder::{icon, use_noun};
use crudkit_leptos::prelude::*;
use leptonic::atoms::prelude::{
    Button, Dialog, DialogDescription, DialogTitle, ModalBackdrop, ModalContent,
};
use leptonic::hooks::DialogRole;
use leptos::prelude::*;

#[component]
fn MarauderConfirm(
    #[prop(into)] is_open: Signal<bool>,
    #[prop(into)] title: Signal<String>,
    #[prop(into)] message: String,
    #[prop(into)] cancel_label: String,
    #[prop(into)] confirm_label: String,
    #[prop(into)] on_confirm: Callback<()>,
    #[prop(into)] on_cancel: Callback<()>,
) -> impl IntoView {
    let message = StoredValue::new(message);
    let (cancel_label, confirm_label) = (
        StoredValue::new(cancel_label),
        StoredValue::new(confirm_label),
    );
    // Escape and outside clicks close the overlay, which cancels.
    let set_open = Callback::new(move |open: bool| {
        if !open {
            on_cancel.run(());
        }
    });
    view! {
        <ModalBackdrop is_open set_open classes="mrd-backdrop">
            <ModalContent classes="mrd-dialog">
                <Dialog role=DialogRole::AlertDialog>
                    <div class="mrd-dialog-body">
                        <span class="mrd-dialog-icon">{icon(icondata::LuTriangleAlert)}</span>
                        <div>
                            <DialogTitle classes="mrd-dialog-title">
                                {move || title.get()}
                            </DialogTitle>
                            <DialogDescription classes="mrd-dialog-message">
                                {message.get_value()}
                            </DialogDescription>
                        </div>
                    </div>
                    <div class="mrd-dialog-actions">
                        <Button classes="mrd-button" on_press=move |_| on_cancel.run(())>
                            {cancel_label.get_value()}
                        </Button>
                        <Button
                            classes=["mrd-button", "mrd-button-danger"]
                            on_press=move |_| on_confirm.run(())
                        >
                            {confirm_label.get_value()}
                        </Button>
                    </div>
                </Dialog>
            </ModalContent>
        </ModalBackdrop>
    }
}

/// Dialogs for the pending deletions and navigation attempts of the surrounding instance.
#[component]
pub fn MarauderDialogs() -> impl IntoView {
    let noun = use_noun();
    let deletion = use_crud_delete();
    let leave = use_crud_leave_confirmation();
    let pending_many = deletion.pending_many;
    let many = Signal::derive(move || {
        pending_many
            .read()
            .as_ref()
            .map_or(0, |entities| entities.len())
    });
    view! {
        <MarauderConfirm
            is_open=Signal::derive(move || deletion.pending.read().is_some())
            title=format!("Delete this {}?", noun.singular)
            message="The record vanishes from the registry for everyone. No spell can undo this."
            cancel_label="Cancel"
            confirm_label="Delete"
            on_confirm=move |()| deletion.confirm()
            on_cancel=move |()| deletion.cancel()
        />
        <MarauderConfirm
            is_open=Signal::derive(move || many.get() > 0)
            title=Signal::derive(move || format!("Delete {} {}?", many.get(), noun.plural))
            message="The selected records vanish from the registry for everyone. No spell can undo this."
            cancel_label="Cancel"
            confirm_label="Delete all"
            on_confirm=move |()| deletion.confirm_many()
            on_cancel=move |()| deletion.cancel_many()
        />
        <MarauderConfirm
            is_open=leave.is_pending
            title="Discard unsaved changes?".to_owned()
            message="You edited this record without saving. Leaving now discards those edits."
            cancel_label="Keep editing"
            confirm_label="Discard"
            on_confirm=move |()| leave.accept()
            on_cancel=move |()| leave.cancel()
        />
    }
}
