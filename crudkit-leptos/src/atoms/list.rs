//! The list of an instance's entities, and atoms acting on its selection.

use crate::atoms::content::content_or_default;
use crate::atoms::status::{StatusRegion, failed_status_text, status_content};
use crate::config::CrudText;
use crate::hooks::delete::use_crud_delete;
use crate::hooks::instance::use_crud_instance;
use crate::hooks::list::{CrudListLoadStatus, expect_crud_list, use_crud_list_state};
use crate::hooks::texts::use_crud_texts;
use leptonic::atoms::button::Button;
use leptonic::leptos_classes::{Classes, MergeStrategy};
use leptonic::leptos_styles::Styles;
use leptos::context::Provider;
use leptos::prelude::*;

/// Loads the current page of the surrounding instance's entities, and provides the list to the
/// atoms inside: [`CrudTable`](crate::atoms::CrudTable),
/// [`CrudPagination`](crate::atoms::CrudPagination), [`CrudListStatus`], and the selection
/// buttons. Renders no element of its own.
///
/// Paging and ordering belong to the instance, so they survive switching views. The selection
/// belongs to this list. Inside, [`use_crud_list`] returns the list.
#[component]
pub fn CrudList(children: Children) -> impl IntoView {
    let list = use_crud_list_state();
    view! { <Provider value=list>{children()}</Provider> }
}

/// Explains why the surrounding list shows no rows: a `<div>` that is a polite live region
/// (`role="status"`), stating that the list loads, is empty, or failed to load, and empty while
/// the list shows rows.
///
/// Default content: the matching prop, else [`CrudText::StatusLoading`],
/// [`CrudText::StatusEmpty`], or [`CrudText::StatusFailed`] with the reason. The list's
/// [`status`](crate::hooks::CrudListState::status) tells the reason too.
///
/// Data attributes: `data-status` (`loading`, `empty`, `failed`, or `ready`).
///
/// Default class: `crudkit-ListStatus`.
///
/// [`CrudText::StatusLoading`]: crate::config::CrudText::StatusLoading
/// [`CrudText::StatusEmpty`]: crate::config::CrudText::StatusEmpty
/// [`CrudText::StatusFailed`]: crate::config::CrudText::StatusFailed
#[component]
pub fn CrudListStatus(
    /// Shown while the list loads.
    #[prop(into, optional)]
    loading: Option<ViewFn>,
    /// Shown when the list has no entities.
    #[prop(into, optional)]
    empty: Option<ViewFn>,
    /// Shown when loading the list failed.
    #[prop(into, optional)]
    failed: Option<ViewFn>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-ListStatus")
        .merge(classes, MergeStrategy::UnionConditions);
    let texts = use_crud_texts();
    let status = expect_crud_list("CrudListStatus").status;
    let data_status = Signal::derive(move || match status.get() {
        CrudListLoadStatus::Loading => "loading",
        CrudListLoadStatus::Empty => "empty",
        CrudListLoadStatus::Failed(_) => "failed",
        CrudListLoadStatus::Ready => "ready",
    });
    let content = move || match status.get() {
        CrudListLoadStatus::Ready => ().into_any(),
        CrudListLoadStatus::Loading => {
            status_content(loading.as_ref(), texts.get(CrudText::StatusLoading))
        }
        CrudListLoadStatus::Empty => {
            status_content(empty.as_ref(), texts.get(CrudText::StatusEmpty))
        }
        CrudListLoadStatus::Failed(error) => {
            status_content(failed.as_ref(), failed_status_text(texts, &error))
        }
    };
    view! {
        <StatusRegion data_status classes styles>
            {content.clone()}
        </StatusRegion>
    }
}

/// Asks to delete the selected entities of the surrounding list. Deleting proceeds once the user
/// confirms it in the instance's [`CrudDeleteManyDialog`](crate::atoms::CrudDeleteManyDialog).
///
/// Disabled while nothing is selected.
///
/// Default content: [`CrudText::SelectionDelete`].
///
/// Data attributes: those of Leptonic's `Button`.
///
/// Default classes: `leptonic-Button crudkit-DeleteSelectedButton`.
///
/// [`CrudText::SelectionDelete`]: crate::config::CrudText::SelectionDelete
#[component]
pub fn CrudDeleteSelectedButton(
    #[prop(into, optional)] is_disabled: Signal<bool>,
    /// Labels the button when its content doesn't, e.g. an icon.
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-DeleteSelectedButton")
        .merge(classes, MergeStrategy::UnionConditions);
    let selection = expect_crud_list("CrudDeleteSelectedButton").selection;
    let deletion = use_crud_delete();
    let texts = use_crud_texts();
    let text = move || texts.get(CrudText::SelectionDelete);
    view! {
        <Button
            on_press=move |_| deletion.request_many(selection.selected())
            is_disabled=Signal::derive(move || selection.count.get() == 0 || is_disabled.get())
            aria_label
            classes
            styles
        >
            {content_or_default(children, text)}
        </Button>
    }
}

/// Deselects every entity of the surrounding list.
///
/// Disabled while nothing is selected.
///
/// Default content: [`CrudText::SelectionClear`].
///
/// Data attributes: those of Leptonic's `Button`.
///
/// Default classes: `leptonic-Button crudkit-ClearSelectionButton`.
///
/// [`CrudText::SelectionClear`]: crate::config::CrudText::SelectionClear
#[component]
pub fn CrudClearSelectionButton(
    #[prop(into, optional)] is_disabled: Signal<bool>,
    /// Labels the button when its content doesn't, e.g. an icon.
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-ClearSelectionButton")
        .merge(classes, MergeStrategy::UnionConditions);
    let selection = expect_crud_list("CrudClearSelectionButton").selection;
    let texts = use_crud_texts();
    let text = move || texts.get(CrudText::SelectionClear);
    view! {
        <Button
            on_press=move |_| selection.clear()
            is_disabled=Signal::derive(move || selection.count.get() == 0 || is_disabled.get())
            aria_label
            classes
            styles
        >
            {content_or_default(children, text)}
        </Button>
    }
}

/// Resets the surrounding instance to its configuration: its first view, page, page size, and
/// ordering. Asks for confirmation while a form has unsaved changes.
///
/// Default content: [`CrudText::InstanceReset`].
///
/// Data attributes: those of Leptonic's `Button`.
///
/// Default classes: `leptonic-Button crudkit-ResetButton`.
///
/// [`CrudText::InstanceReset`]: crate::config::CrudText::InstanceReset
#[component]
pub fn CrudResetButton(
    #[prop(into, optional)] is_disabled: Signal<bool>,
    /// Labels the button when its content doesn't, e.g. an icon.
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-ResetButton")
        .merge(classes, MergeStrategy::UnionConditions);
    let ctx = use_crud_instance();
    let texts = use_crud_texts();
    let text = move || texts.get(CrudText::InstanceReset);
    view! {
        <Button on_press=move |_| ctx.reset() is_disabled aria_label classes styles>
            {content_or_default(children, text)}
        </Button>
    }
}

/// The number of selected entities of the surrounding list: a `<span>`, e.g. for a toolbar. It is
/// a polite live region, so screen readers announce selection changes.
///
/// Content: [`CrudText::SelectionCount`] of the number.
///
/// Data attributes: `data-empty` (nothing is selected).
///
/// Default class: `crudkit-SelectionCount`.
///
/// [`CrudText::SelectionCount`]: crate::config::CrudText::SelectionCount
#[component]
pub fn CrudSelectionCount(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-SelectionCount")
        .merge(classes, MergeStrategy::UnionConditions);
    let count = expect_crud_list("CrudSelectionCount").selection.count;
    let texts = use_crud_texts();
    let text = move || {
        texts.get(CrudText::SelectionCount {
            count: count.get() as u64,
        })
    };
    view! {
        <span
            class=classes
            style=styles
            role="status"
            data-empty=move || (count.get() == 0).then_some("true")
        >
            {text}
        </span>
    }
}
