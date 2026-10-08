//! Buttons running the instance's configured actions.

use crate::atoms::content::content_or_default;
use crate::atoms::icon::CrudIcon;
use crate::config::{CrudAction, CrudActionIntent, CrudEntityAction};
use crate::hooks::actions::{CrudActionHandle, use_crud_actions_state};
use crate::hooks::form::expect_crud_form;
use crate::hooks::instance::use_crud_instance;
use leptonic::atoms::prelude::Button;
use leptonic::utils::classes::{Classes, MergeStrategy};
use leptonic::utils::styles::Styles;
use leptos::prelude::*;

/// A [`CrudActionButton`] per resource action of the surrounding instance (its
/// [`actions`](crate::config::CrudInstanceConfig::actions)). Renders no element of its own.
#[component]
pub fn CrudResourceActions(
    /// Classes of each button.
    #[prop(into, optional)]
    classes: Classes,
) -> impl IntoView {
    let actions = use_crud_instance().resource_actions();
    actions
        .into_iter()
        .map(|action| view! { <CrudActionButton action classes=classes.clone() /> })
        .collect_view()
}

/// A [`CrudEntityActionButton`] per entity action of the surrounding instance (its
/// [`entity_actions`](crate::config::CrudInstanceConfig::entity_actions)) offered in the
/// surrounding form. Renders no element of its own.
#[component]
pub fn CrudEntityActions(
    /// Classes of each button.
    #[prop(into, optional)]
    classes: Classes,
) -> impl IntoView {
    let form = expect_crud_form("CrudEntityActions");
    let actions = use_crud_instance().entity_actions(form.kind);
    actions
        .into_iter()
        .map(|action| view! { <CrudEntityActionButton action classes=classes.clone() /> })
        .collect_view()
}

/// Runs the resource `action`. An action with a view shows it first, e.g. a dialog asking for its
/// payload; the view is an overlay, such as a Leptonic `ModalBackdrop`. Disabled while the action
/// runs.
///
/// Default content: the action's icon, if it has one, as a [`CrudIcon`], and its name.
///
/// Data attributes: `data-intent` (`primary`, `secondary`, `success`, `warning`, or `danger`), and
/// those of Leptonic's `Button`.
///
/// Default classes: `leptonic-Button crudkit-ActionButton`.
#[component]
pub fn CrudActionButton(
    /// The action to run.
    action: CrudAction,
    #[prop(into, optional)] is_disabled: Signal<bool>,
    /// Labels the button when its content doesn't, e.g. an icon.
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-ActionButton")
        .merge(classes, MergeStrategy::UnionConditions);
    let handle = use_crud_actions_state().bind_resource_action(&action);
    view! {
        <ActionButton
            handle
            intent=action.intent
            icon=action.icon
            name=action.name
            is_disabled
            aria_label
            classes
            styles
            children
        />
    }
}

/// Runs the entity `action` on the entity of the surrounding form, including its unsaved changes.
/// An action with a view shows it first, e.g. a dialog asking for its payload; the view is an
/// overlay, such as a Leptonic `ModalBackdrop`. Disabled while the action runs or the form has no
/// entity.
///
/// Default content: the action's icon, if it has one, as a [`CrudIcon`], and its name.
///
/// Data attributes: `data-intent` (`primary`, `secondary`, `success`, `warning`, or `danger`), and
/// those of Leptonic's `Button`.
///
/// Default classes: `leptonic-Button crudkit-EntityActionButton`.
#[component]
pub fn CrudEntityActionButton(
    /// The action to run.
    action: CrudEntityAction,
    #[prop(into, optional)] is_disabled: Signal<bool>,
    /// Labels the button when its content doesn't, e.g. an icon.
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-EntityActionButton")
        .merge(classes, MergeStrategy::UnionConditions);
    let draft = expect_crud_form("CrudEntityActionButton").draft;
    let handle = use_crud_actions_state().bind_entity_action(&action, draft);
    view! {
        <ActionButton
            handle
            intent=action.intent
            icon=action.icon
            name=action.name
            is_disabled
            aria_label
            classes
            styles
            children
        />
    }
}

/// The button running an action through `handle`, followed by the action's view. Shared by the
/// action button atoms.
#[component]
fn ActionButton(
    handle: CrudActionHandle,
    intent: CrudActionIntent,
    icon: Option<icondata_core::Icon>,
    name: String,
    is_disabled: Signal<bool>,
    aria_label: MaybeProp<String>,
    classes: Classes,
    styles: Styles,
    children: Option<Children>,
) -> impl IntoView {
    let CrudActionHandle {
        press,
        is_disabled: is_unavailable,
        view,
    } = handle;
    view! {
        <Button
            on_press=move |_| press.run(())
            is_disabled=Signal::derive(move || is_unavailable.get() || is_disabled.get())
            aria_label
            classes
            styles
            attr:data-intent=data_intent(intent)
        >
            {content_or_default(children, view! { <ActionLabel icon name /> })}
        </Button>
        {view}
    }
}

/// The default content of an action button: the action's icon and name.
#[component]
fn ActionLabel(icon: Option<icondata_core::Icon>, name: String) -> impl IntoView {
    view! {
        {icon.map(|icon| view! { <CrudIcon icon /> })}
        {name}
    }
}

/// The `data-intent` value of `intent`.
fn data_intent(intent: CrudActionIntent) -> &'static str {
    match intent {
        CrudActionIntent::Primary => "primary",
        CrudActionIntent::Secondary => "secondary",
        CrudActionIntent::Success => "success",
        CrudActionIntent::Warning => "warning",
        CrudActionIntent::Danger => "danger",
    }
}
