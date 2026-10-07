//! Visible notifications.

use crate::components::button::CrudButton;
use crate::components::with_classes;
use crate::hooks::notify::{CrudNotification, CrudNotificationKind, CrudNotificationQueue};
use crate::hooks::texts::use_crud_texts;
use leptonic::utils::classes::Classes;
use leptos::prelude::*;

fn kind_name(kind: CrudNotificationKind) -> &'static str {
    match kind {
        CrudNotificationKind::Success => "success",
        CrudNotificationKind::Info => "info",
        CrudNotificationKind::Warning => "warning",
        CrudNotificationKind::Error => "error",
    }
}

/// Lists the notifications of `queue` in a polite live region.
#[component]
pub fn CrudNotificationRegion(
    /// The queue whose entries are listed. Dismissing an entry removes it from the queue.
    queue: CrudNotificationQueue,
    /// Additional classes of the root element.
    #[prop(into, optional)]
    classes: Classes,
) -> impl IntoView {
    let dismiss_label = StoredValue::new(use_crud_texts().dismiss.to_string());
    view! {
        <div class=with_classes("crudkit-notifications", classes) role="status" aria-live="polite">
            <For
                each=move || queue.entries.get()
                key=|(id, _)| *id
                children=move |(id, CrudNotification { kind, title, message })| {
                    view! {
                        <div class="crudkit-notification" data-kind=kind_name(kind)>
                            <div class="crudkit-notification-title">{title}</div>
                            <div class="crudkit-notification-message">{message}</div>
                            <CrudButton on_press=move |_| {
                                queue.dismiss(id);
                            }>{dismiss_label.get_value()}</CrudButton>
                        </div>
                    }
                }
            />
        </div>
    }
}
