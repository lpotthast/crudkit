//! Buttons triggering application actions.

use crate::atoms::icon::CrudIcon;
use crate::components::button::CrudButton;
use crate::config::{CrudActionIntent, CrudEntityViewKind};
use crate::hooks::actions::{CrudActionHandle, use_crud_actions};
use crudkit_web::prelude::DynUpdateModel;
use leptos::prelude::*;

/// Renders `handle` as a button, followed by the action's view.
fn action_button(
    handle: CrudActionHandle,
    name: String,
    icon: Option<icondata::Icon>,
    intent: CrudActionIntent,
) -> impl IntoView {
    let CrudActionHandle {
        press,
        is_disabled,
        view,
    } = handle;
    view! {
        <CrudButton intent is_disabled on_press=move |_| press.run(())>
            {icon.map(|icon| view! { <CrudIcon icon classes="crudkit-icon" /> })}
            {name}
        </CrudButton>
        {view}
    }
}

/// Buttons for the instance's resource actions, followed by the views of requested actions.
#[component]
pub fn CrudResourceActionButtons() -> impl IntoView {
    let actions = use_crud_actions();
    actions
        .resource_actions()
        .into_iter()
        .map(|action| {
            let handle = actions.bind_resource_action(&action);
            action_button(handle, action.name, action.icon, action.intent)
        })
        .collect_view()
}

/// Buttons for the entity actions available in `required_state`, acting on `input`.
#[component]
pub fn CrudEntityActionButtons(
    /// The entity the actions act on, including unsaved changes. The buttons are disabled while
    /// it is `None`.
    #[prop(into)]
    input: Signal<Option<DynUpdateModel>>,
    /// The kind of the surrounding view. Only actions valid in it are shown.
    required_state: CrudEntityViewKind,
) -> impl IntoView {
    let actions = use_crud_actions();
    actions
        .entity_actions(required_state)
        .into_iter()
        .map(|action| {
            let handle = actions.bind_entity_action(&action, input);
            action_button(handle, action.name, action.icon, action.intent)
        })
        .collect_view()
}
