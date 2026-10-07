//! Confirmation dialogs.

use crate::components::button::CrudButton;
use crate::components::with_classes;
use crate::config::CrudActionIntent;
use crate::hooks::delete::use_crud_delete;
use crate::hooks::leave::use_crud_leave_confirmation;
use crate::hooks::texts::use_crud_texts;
use leptonic::atoms::prelude::{
    Dialog, DialogDescription, DialogTitle, ModalBackdrop, ModalContent,
};
use leptonic::hooks::DialogRole;
use leptonic::utils::classes::Classes;
use leptos::prelude::*;

/// A modal alert dialog asking the user to confirm an action.
///
/// Escape and the cancel button cancel. Focus is contained in the dialog and restored afterwards.
#[component]
pub fn CrudConfirmDialog(
    /// Whether the dialog is shown. The dialog does not close itself; the owner of this signal
    /// closes it in `on_confirm` and `on_cancel`.
    #[prop(into)]
    is_open: Signal<bool>,
    /// Heading of the dialog, also its accessible name.
    #[prop(into)]
    title: Signal<String>,
    /// Text explaining what is confirmed, also the dialog's accessible description.
    #[prop(into)]
    message: Signal<String>,
    /// Label of the confirm button.
    #[prop(into)]
    confirm_label: Signal<String>,
    /// Intent of the confirm button. Defaults to [`CrudActionIntent::Secondary`].
    #[prop(optional)]
    confirm_intent: CrudActionIntent,
    /// Called when the confirm button is pressed.
    #[prop(into)]
    on_confirm: Callback<()>,
    /// Called when the cancel button is pressed, on Escape, and on a press outside the dialog.
    #[prop(into)]
    on_cancel: Callback<()>,
    /// Disables confirming, e.g. while the confirmed operation is in flight.
    #[prop(into, optional)]
    is_busy: Signal<bool>,
    /// Additional classes of the dialog.
    #[prop(into, optional)]
    classes: Classes,
) -> impl IntoView {
    let cancel_label = StoredValue::new(use_crud_texts().cancel.to_string());
    let classes = with_classes("crudkit-dialog", classes);
    // Escape and outside clicks close the overlay, which cancels.
    let set_open = Callback::new(move |open: bool| {
        if !open {
            on_cancel.run(());
        }
    });
    // The content mounts only while the dialog is open. It is created once per opening, so that
    // focus stays where it is; only the texts update in place.
    view! {
        <ModalBackdrop is_open set_open classes="crudkit-dialog-backdrop">
            <ModalContent classes=classes.clone()>
                <Dialog role=DialogRole::AlertDialog>
                    <DialogTitle classes="crudkit-dialog-title">{move || title.get()}</DialogTitle>
                    <DialogDescription classes="crudkit-dialog-message">
                        {move || message.get()}
                    </DialogDescription>
                    <div class="crudkit-dialog-actions">
                        <CrudButton on_press=move |_| {
                            on_cancel.run(());
                        }>{cancel_label.get_value()}</CrudButton>
                        <CrudButton
                            intent=confirm_intent
                            is_disabled=is_busy
                            on_press=move |_| on_confirm.run(())
                        >
                            {move || confirm_label.get()}
                        </CrudButton>
                    </div>
                </Dialog>
            </ModalContent>
        </ModalBackdrop>
    }
}

/// Confirmation dialogs for the pending deletions and navigation attempts of the surrounding
/// instance.
#[component]
pub fn CrudInstanceDialogs() -> impl IntoView {
    let texts = use_crud_texts();
    let deletion = use_crud_delete();
    let leave = use_crud_leave_confirmation();
    let pending = deletion.pending;
    let pending_many = deletion.pending_many;
    let count =
        Signal::derive(move || pending_many.read().as_ref().map_or(0, |it| it.len() as u64));
    let (t1, t2, t3) = (texts.clone(), texts.clone(), texts.clone());
    view! {
        <CrudConfirmDialog
            is_open=Signal::derive(move || pending.read().is_some())
            title=texts.delete_title.to_string()
            message=texts.delete_body.to_string()
            confirm_label=texts.delete.to_string()
            confirm_intent=CrudActionIntent::Danger
            on_confirm=move |()| deletion.confirm()
            is_busy=deletion.is_deleting
            on_cancel=move |()| deletion.cancel()
        />
        <CrudConfirmDialog
            is_open=Signal::derive(move || pending_many.read().is_some())
            title=Signal::derive(move || (t1.delete_many_title)(count.get()))
            message=Signal::derive(move || (t2.delete_many_body)(count.get()))
            confirm_label=Signal::derive(move || (t3.delete_many_title)(count.get()))
            confirm_intent=CrudActionIntent::Danger
            on_confirm=move |()| deletion.confirm_many()
            is_busy=deletion.is_deleting
            on_cancel=move |()| deletion.cancel_many()
        />
        <CrudConfirmDialog
            is_open=leave.is_pending
            title=texts.leave_title.to_string()
            message=texts.leave_body.to_string()
            confirm_label=texts.leave.to_string()
            confirm_intent=CrudActionIntent::Danger
            on_confirm=move |()| leave.accept()
            on_cancel=move |()| leave.cancel()
        />
    }
}
