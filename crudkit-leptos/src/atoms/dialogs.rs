//! Dialogs confirming deletions and leaving views with unsaved changes.

use crate::atoms::content::content_or_default;
use crate::config::CrudText;
use crate::hooks::delete::use_crud_delete;
use crate::hooks::instance::use_crud_instance;
use crate::hooks::leave::use_crud_leave_confirmation;
use crate::hooks::texts::use_crud_texts;
use crate::instance::MountedDialogs;
use leptonic::HeadingLevel;
use leptonic::atoms::button::Button;
use leptonic::atoms::dialog::{DialogDescription, DialogTitle};
use leptonic::atoms::modal::ModalBackdrop;
use leptonic::leptos_classes::{Classes, MergeStrategy};
use leptonic::leptos_styles::Styles;
use leptos::context::Provider;
use leptos::prelude::*;

/// What a confirmation dialog confirms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfirmationKind {
    Delete,
    DeleteMany,
    Leave,
}

/// The pending confirmation of the surrounding dialog, for its title, message, and buttons.
#[derive(Clone, Copy)]
struct Confirmation {
    kind: ConfirmationKind,
    /// The number of entities to delete; one outside of [`CrudDeleteManyDialog`].
    count: Signal<usize>,
    /// Whether the confirmed operation is running, e.g. a deletion request.
    is_busy: Signal<bool>,
    confirm: Callback<()>,
    cancel: Callback<()>,
}

impl Confirmation {
    /// The question the dialog asks.
    fn title(&self) -> CrudText<'static> {
        match self.kind {
            ConfirmationKind::Delete => CrudText::DeleteDialogTitle,
            ConfirmationKind::DeleteMany => CrudText::DeleteManyDialogTitle {
                count: self.count.get() as u64,
            },
            ConfirmationKind::Leave => CrudText::LeaveDialogTitle,
        }
    }

    /// What confirming means.
    fn message(&self) -> CrudText<'static> {
        match self.kind {
            ConfirmationKind::Delete => CrudText::DeleteDialogMessage,
            ConfirmationKind::DeleteMany => CrudText::DeleteManyDialogMessage {
                count: self.count.get() as u64,
            },
            ConfirmationKind::Leave => CrudText::LeaveDialogMessage,
        }
    }

    /// What the confirm button does.
    fn confirm(&self) -> CrudText<'static> {
        match self.kind {
            ConfirmationKind::Delete | ConfirmationKind::DeleteMany => CrudText::EntityDelete,
            ConfirmationKind::Leave => CrudText::LeaveDialogConfirm,
        }
    }

    /// Cancels when the modal closes, e.g. on Escape or a press outside of it. Stays open while
    /// the confirmed operation runs.
    fn set_open(self) -> Callback<bool> {
        Callback::new(move |open: bool| {
            if !open && !self.is_busy.get_untracked() {
                self.cancel.run(());
            }
        })
    }
}

/// Confirms deleting the entity a [`CrudDeleteButton`](crate::atoms::CrudDeleteButton) asked to
/// delete: a Leptonic `ModalBackdrop`, open while the deletion awaits confirmation. Pressing Escape
/// or outside of it cancels.
///
/// Compose it from Leptonic's `ModalContent` and `Dialog` (with
/// `role=DialogRole::AlertDialog`), a [`CrudConfirmationTitle`], a [`CrudConfirmationMessage`],
/// a [`CrudCancelButton`], and a [`CrudConfirmButton`]. Place it once per instance, next to its
/// [`CrudViewOutlet`](crate::instance::CrudViewOutlet).
///
/// Data attributes: those of Leptonic's `ModalBackdrop`.
///
/// Default classes: `leptonic-ModalBackdrop crudkit-DeleteDialog`.
#[component]
pub fn CrudDeleteDialog(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: ChildrenFn,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-DeleteDialog")
        .merge(classes, MergeStrategy::UnionConditions);
    MountedDialogs::register(use_crud_instance().dialogs.delete);
    let deletion = use_crud_delete();
    let confirmation = Confirmation {
        kind: ConfirmationKind::Delete,
        count: Signal::stored(1),
        is_busy: deletion.is_deleting,
        confirm: Callback::new(move |()| deletion.confirm()),
        cancel: Callback::new(move |()| deletion.cancel()),
    };
    view! {
        <Provider value=confirmation>
            <ModalBackdrop
                is_open=Signal::derive(move || deletion.pending.read().is_some())
                set_open=confirmation.set_open()
                is_dismissable=true
                classes
                styles
            >
                {children()}
            </ModalBackdrop>
        </Provider>
    }
}

/// Confirms deleting the entities a
/// [`CrudDeleteSelectedButton`](crate::atoms::CrudDeleteSelectedButton) asked to delete: a
/// Leptonic `ModalBackdrop`, open while the deletion awaits confirmation. Pressing Escape or outside
/// of it cancels.
///
/// Compose it like a [`CrudDeleteDialog`]; its [`CrudConfirmationTitle`] and
/// [`CrudConfirmationMessage`] name the number of entities.
///
/// Data attributes: those of Leptonic's `ModalBackdrop`.
///
/// Default classes: `leptonic-ModalBackdrop crudkit-DeleteManyDialog`.
#[component]
pub fn CrudDeleteManyDialog(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: ChildrenFn,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-DeleteManyDialog")
        .merge(classes, MergeStrategy::UnionConditions);
    MountedDialogs::register(use_crud_instance().dialogs.delete_many);
    let deletion = use_crud_delete();
    let confirmation = Confirmation {
        kind: ConfirmationKind::DeleteMany,
        // The last count stays while the dialog animates out, after the deletion stopped pending.
        count: Memo::new(move |previous: Option<&usize>| {
            deletion.pending_many.read().as_ref().map_or_else(
                || previous.copied().unwrap_or_default(),
                |entities| entities.len(),
            )
        })
        .into(),
        is_busy: deletion.is_deleting,
        confirm: Callback::new(move |()| deletion.confirm_many()),
        cancel: Callback::new(move |()| deletion.cancel_many()),
    };
    view! {
        <Provider value=confirmation>
            <ModalBackdrop
                is_open=Signal::derive(move || deletion.pending_many.read().is_some())
                set_open=confirmation.set_open()
                is_dismissable=true
                classes
                styles
            >
                {children()}
            </ModalBackdrop>
        </Provider>
    }
}

/// Confirms leaving a view whose forms have unsaved changes: a Leptonic `ModalBackdrop`, open
/// while navigating away awaits confirmation. Confirming discards the changes. Pressing Escape or
/// outside of it cancels, keeping the view.
///
/// Compose it like a [`CrudDeleteDialog`].
///
/// Data attributes: those of Leptonic's `ModalBackdrop`.
///
/// Default classes: `leptonic-ModalBackdrop crudkit-LeaveDialog`.
#[component]
pub fn CrudLeaveDialog(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: ChildrenFn,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-LeaveDialog")
        .merge(classes, MergeStrategy::UnionConditions);
    MountedDialogs::register(use_crud_instance().dialogs.leave);
    let leave = use_crud_leave_confirmation();
    let confirmation = Confirmation {
        kind: ConfirmationKind::Leave,
        count: Signal::stored(1),
        is_busy: Signal::stored(false),
        confirm: Callback::new(move |()| leave.accept()),
        cancel: Callback::new(move |()| leave.cancel()),
    };
    view! {
        <Provider value=confirmation>
            <ModalBackdrop
                is_open=leave.is_pending
                set_open=confirmation.set_open()
                is_dismissable=true
                classes
                styles
            >
                {children()}
            </ModalBackdrop>
        </Provider>
    }
}

/// Returns the confirmation of the surrounding dialog.
///
/// # Panics
///
/// Panics when `atom` is rendered outside of a confirmation dialog.
fn use_confirmation(atom: &'static str) -> Confirmation {
    use_context::<Confirmation>().unwrap_or_else(|| {
        panic!("`{atom}` must be rendered inside a confirmation dialog, e.g. a `CrudDeleteDialog`")
    })
}

/// The title of the surrounding confirmation dialog: a Leptonic `DialogTitle` inside its `Dialog`.
///
/// Default content: [`CrudText::DeleteDialogTitle`], [`CrudText::DeleteManyDialogTitle`] (given
/// the number of entities), or [`CrudText::LeaveDialogTitle`].
///
/// Default classes: `leptonic-DialogTitle crudkit-ConfirmationTitle`.
///
/// [`CrudText::DeleteDialogTitle`]: crate::config::CrudText::DeleteDialogTitle
/// [`CrudText::DeleteManyDialogTitle`]: crate::config::CrudText::DeleteManyDialogTitle
/// [`CrudText::LeaveDialogTitle`]: crate::config::CrudText::LeaveDialogTitle
#[component]
pub fn CrudConfirmationTitle(
    /// The heading level. Defaults to `h2`.
    #[prop(optional)]
    level: HeadingLevel,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-ConfirmationTitle")
        .merge(classes, MergeStrategy::UnionConditions);
    let confirmation = use_confirmation("CrudConfirmationTitle");
    let texts = use_crud_texts();
    let text = move || texts.get(confirmation.title());
    view! {
        <DialogTitle level classes styles>
            {content_or_default(children, text)}
        </DialogTitle>
    }
}

/// The message of the surrounding confirmation dialog: a Leptonic `DialogDescription` inside its
/// `Dialog`.
///
/// Default content: [`CrudText::DeleteDialogMessage`], [`CrudText::DeleteManyDialogMessage`]
/// (given the number of entities), or [`CrudText::LeaveDialogMessage`].
///
/// Default classes: `leptonic-DialogDescription crudkit-ConfirmationMessage`.
///
/// [`CrudText::DeleteDialogMessage`]: crate::config::CrudText::DeleteDialogMessage
/// [`CrudText::DeleteManyDialogMessage`]: crate::config::CrudText::DeleteManyDialogMessage
/// [`CrudText::LeaveDialogMessage`]: crate::config::CrudText::LeaveDialogMessage
#[component]
pub fn CrudConfirmationMessage(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-ConfirmationMessage")
        .merge(classes, MergeStrategy::UnionConditions);
    let confirmation = use_confirmation("CrudConfirmationMessage");
    let texts = use_crud_texts();
    let text = move || texts.get(confirmation.message());
    view! {
        <DialogDescription classes styles>
            {content_or_default(children, text)}
        </DialogDescription>
    }
}

/// Confirms the surrounding dialog: deletes, or leaves the view. Pending while the deletion runs.
///
/// Default content: [`CrudText::EntityDelete`] in delete dialogs, [`CrudText::LeaveDialogConfirm`]
/// in the leave dialog.
///
/// Data attributes: those of Leptonic's `Button`, e.g. `data-pending`.
///
/// Default classes: `leptonic-Button crudkit-ConfirmButton`.
///
/// [`CrudText::EntityDelete`]: crate::config::CrudText::EntityDelete
/// [`CrudText::LeaveDialogConfirm`]: crate::config::CrudText::LeaveDialogConfirm
#[component]
pub fn CrudConfirmButton(
    #[prop(into, optional)] is_disabled: Signal<bool>,
    /// Labels the button when its content doesn't, e.g. an icon.
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-ConfirmButton")
        .merge(classes, MergeStrategy::UnionConditions);
    let confirmation = use_confirmation("CrudConfirmButton");
    let texts = use_crud_texts();
    let text = move || texts.get(confirmation.confirm());
    view! {
        <Button
            on_press=move |_| confirmation.confirm.run(())
            is_disabled
            is_pending=confirmation.is_busy
            aria_label
            classes
            styles
        >
            {content_or_default(children, text)}
        </Button>
    }
}

/// Cancels the surrounding dialog, keeping the entities or the view. Disabled while the deletion
/// runs.
///
/// Default content: [`CrudText::DialogCancel`].
///
/// Data attributes: those of Leptonic's `Button`.
///
/// Default classes: `leptonic-Button crudkit-CancelButton`.
///
/// [`CrudText::DialogCancel`]: crate::config::CrudText::DialogCancel
#[component]
pub fn CrudCancelButton(
    #[prop(into, optional)] is_disabled: Signal<bool>,
    /// Labels the button when its content doesn't, e.g. an icon.
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-CancelButton")
        .merge(classes, MergeStrategy::UnionConditions);
    let confirmation = use_confirmation("CrudCancelButton");
    let texts = use_crud_texts();
    let text = move || texts.get(CrudText::DialogCancel);
    view! {
        <Button
            on_press=move |_| confirmation.cancel.run(())
            is_disabled=Signal::derive(move || confirmation.is_busy.get() || is_disabled.get())
            aria_label
            classes
            styles
        >
            {content_or_default(children, text)}
        </Button>
    }
}
