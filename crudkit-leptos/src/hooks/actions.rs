//! Resource and entity actions.

use crate::config::{
    CrudAction, CrudEntityAction, CrudEntityViewKind, EntityActionInput, EntityActionViewInput,
    ResourceActionInput, ResourceActionViewInput,
};
use crate::hooks::instance::use_crud_instance;
use crate::instance::CrudInstanceContext;
use crudkit_web::prelude::*;
use leptos::prelude::*;

/// Identifies an action within its instance.
pub type ActionId = &'static str;

/// Requested and executing actions of one view.
///
/// An action with a view is first *requested*, which shows its view, e.g. a modal asking for a
/// payload. Executing an action runs it until it reports its outcome through its `and_then`
/// callback. While executing, the action cannot be triggered again.
#[derive(Debug, Clone, Copy)]
pub struct CrudActionsState {
    instance: CrudInstanceContext,
    requested: RwSignal<Vec<ActionId>>,
    executing: RwSignal<Vec<ActionId>>,
}

/// One action bound to a [`CrudActionsState`], ready to be rendered as a button.
pub struct CrudActionHandle {
    /// Shows the action's view if it has one, and runs the action otherwise.
    pub press: Callback<()>,
    /// Whether the action cannot be pressed, e.g. while it executes.
    pub is_disabled: Signal<bool>,
    /// The action's view, e.g. a modal asking for a payload. Render it next to the button. It shows
    /// itself while the action is requested.
    pub view: Option<AnyView>,
}

impl std::fmt::Debug for CrudActionHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CrudActionHandle")
            .field("is_disabled", &self.is_disabled)
            .field("has_view", &self.view.is_some())
            .finish_non_exhaustive()
    }
}

/// Creates action state for a view of the surrounding instance.
///
/// # Panics
///
/// Panics when called outside of a CrudKit instance.
#[must_use]
pub fn use_crud_actions() -> CrudActionsState {
    CrudActionsState {
        instance: use_crud_instance(),
        requested: RwSignal::new(Vec::new()),
        executing: RwSignal::new(Vec::new()),
    }
}

impl CrudActionsState {
    /// Binds the resource `action` to this state.
    #[must_use]
    pub fn bind_resource_action(&self, action: &CrudAction) -> CrudActionHandle {
        let this = *self;
        let id = action.id;
        let has_view = action.view.is_some();
        let stored = StoredValue::new(action.clone());
        let view = action.view.map(|view| {
            view.run(ResourceActionViewInput {
                show_when: Signal::derive(move || this.is_requested(id)),
                cancel: Callback::new(move |()| this.cancel(id)),
                execute: Callback::new(move |payload| this.trigger(&stored.read_value(), payload)),
            })
        });
        CrudActionHandle {
            press: Callback::new(move |()| {
                if has_view {
                    this.request(id);
                } else {
                    this.trigger(&stored.read_value(), None);
                }
            }),
            is_disabled: Signal::derive(move || this.is_executing(id)),
            view,
        }
    }

    /// Binds the entity `action`, acting on `input`, to this state. The action is disabled while
    /// there is no input.
    #[must_use]
    pub fn bind_entity_action(
        &self,
        action: &CrudEntityAction,
        input: Signal<Option<DynUpdateModel>>,
    ) -> CrudActionHandle {
        let this = *self;
        let id = action.id;
        let has_view = action.view.is_some();
        let stored = StoredValue::new(action.clone());
        let execute = move |payload| {
            if let Some(model) = input.get_untracked() {
                this.trigger_entity(&stored.read_value(), model, payload);
            }
        };
        let view = action.view.map(|view| {
            view.run(EntityActionViewInput {
                show_when: Signal::derive(move || this.is_requested(id)),
                state: input,
                cancel: Callback::new(move |()| this.cancel(id)),
                execute: Callback::new(execute),
            })
        });
        CrudActionHandle {
            press: Callback::new(move |()| {
                if has_view {
                    this.request(id);
                } else {
                    execute(None);
                }
            }),
            is_disabled: Signal::derive(move || this.is_executing(id) || input.read().is_none()),
            view,
        }
    }

    /// Returns the instance's resource actions.
    #[must_use]
    pub fn resource_actions(&self) -> Vec<CrudAction> {
        self.instance.static_config.read_value().actions.clone()
    }

    /// Returns the instance's entity actions available in `state`.
    #[must_use]
    pub fn entity_actions(&self, state: CrudEntityViewKind) -> Vec<CrudEntityAction> {
        self.instance
            .static_config
            .read_value()
            .entity_actions
            .iter()
            .filter(|action| action.valid_in.contains(&state))
            .cloned()
            .collect()
    }

    /// Requests `action_id`, showing its view.
    pub fn request(&self, action_id: ActionId) {
        tracing::debug!(action_id, "request_action");
        self.requested.update(|actions| actions.push(action_id));
    }

    /// Withdraws the request for `action_id`.
    pub fn cancel(&self, action_id: ActionId) {
        tracing::debug!(action_id, "cancel_action");
        self.requested.update(|actions| remove(actions, action_id));
    }

    /// Returns whether `action_id` is requested. Tracks the requests.
    #[must_use]
    pub fn is_requested(&self, action_id: ActionId) -> bool {
        self.requested.read().contains(&action_id)
    }

    /// Returns whether `action_id` is executing. Tracks the executions.
    #[must_use]
    pub fn is_executing(&self, action_id: ActionId) -> bool {
        self.executing.read().contains(&action_id)
    }

    /// Executes the resource `action`.
    pub fn trigger(&self, action: &CrudAction, payload: Option<DynActionPayload>) {
        tracing::debug!(action_id = action.id, ?payload, "trigger_action");
        let and_then = self.start(action.id);
        action.action.run(ResourceActionInput { payload, and_then });
    }

    /// Executes the entity `action` on `update_model`, which includes unsaved changes.
    pub fn trigger_entity(
        &self,
        action: &CrudEntityAction,
        update_model: DynUpdateModel,
        payload: Option<DynActionPayload>,
    ) {
        tracing::debug!(action_id = action.id, ?payload, "trigger_entity_action");
        let and_then = self.start(action.id);
        action.action.run(EntityActionInput {
            update_model,
            payload,
            and_then,
        });
    }

    /// Marks `action_id` as executing and returns the callback reporting its outcome.
    fn start(
        &self,
        action_id: ActionId,
    ) -> Callback<Result<crate::config::CrudActionAftermath, crate::config::CrudActionAftermath>>
    {
        // The user accepted the request. The action is no longer requested.
        self.requested.update(|actions| remove(actions, action_id));
        self.executing.update(|actions| actions.push(action_id));

        let this = *self;
        Callback::new(move |outcome| {
            tracing::debug!(?outcome, "action finished");
            this.executing.update(|actions| remove(actions, action_id));
            this.instance.handle_action_outcome(outcome);
        })
    }
}

fn remove(actions: &mut Vec<ActionId>, action_id: ActionId) {
    if let Some(position) = actions.iter().position(|id| *id == action_id) {
        actions.remove(position);
    }
}
