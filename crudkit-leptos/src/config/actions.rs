use crate::hooks::notify::CrudNotification;
use crate::prelude::*;
use crudkit_web::prelude::*;
use leptos::prelude::*;
use std::fmt::Debug;
use std::sync::Arc;

/// The kind of view an entity action is offered in.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CrudEntityViewKind {
    /// A create form, editing a new entity, which has no entity to act on yet.
    Create,
    /// An edit view, editing an existing entity.
    Update,
    /// A read view, showing an existing entity without editing it.
    Read,
}

/// Input of the view of a [`CrudAction`], rendered once when the action is bound.
#[derive(Clone, Copy)]
pub struct ResourceActionViewInput {
    /// Whether the action is requested. The view shows itself while this is `true`.
    pub show_when: Signal<bool>,
    /// Withdraws the request without executing the action.
    pub cancel: Callback<()>,
    /// Executes the action with the given payload, ending the request.
    pub execute: Callback<Option<DynActionPayload>>,
}

/// Input of the view of a [`CrudEntityAction`], rendered once when the action is bound.
#[derive(Clone, Copy)]
pub struct EntityActionViewInput {
    /// Whether the action is requested. The view shows itself while this is `true`.
    pub show_when: Signal<bool>,
    /// The entity the action acts on, including unsaved changes. `None` while no entity is
    /// available, e.g. while it loads.
    pub state: Signal<Option<DynUpdateModel>>,
    /// Withdraws the request without executing the action.
    pub cancel: Callback<()>,
    /// Executes the action on the current entity with the given payload. Does nothing while
    /// `state` is `None`.
    pub execute: Callback<Option<DynActionPayload>>,
}

/// Reports the outcome of an action. Must be run exactly once, after the action finished. Until
/// then, the action counts as executing and its buttons are disabled.
///
/// Unlike a Leptos `Callback`, it stays valid when the view that started the action is gone, so
/// an action may finish after the user left the view.
#[derive(Clone)]
pub struct CrudActionCompletion(
    Arc<dyn Fn(Result<CrudActionAftermath, CrudActionAftermath>) + Send + Sync>,
);

impl CrudActionCompletion {
    pub(crate) fn new(
        complete: impl Fn(Result<CrudActionAftermath, CrudActionAftermath>) + Send + Sync + 'static,
    ) -> Self {
        Self(Arc::new(complete))
    }

    /// Reports `outcome`. Both variants apply their aftermath; the variant records whether the
    /// action succeeded.
    pub fn run(&self, outcome: Result<CrudActionAftermath, CrudActionAftermath>) {
        (self.0)(outcome);
    }
}

impl Debug for CrudActionCompletion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CrudActionCompletion")
            .finish_non_exhaustive()
    }
}

/// The concrete data to perform a resource action with.
pub struct ResourceActionInput {
    /// The payload produced by the action's view, or `None` for actions without a view.
    pub payload: Option<DynActionPayload>,
    /// Reports the action's outcome, see [`CrudActionCompletion`].
    pub and_then: CrudActionCompletion,
}

/// The concrete data to perform an entity-action with.
pub struct EntityActionInput {
    /// The current state of this entity when the action is invoked. This includes all unsaved
    /// changes.
    pub update_model: DynUpdateModel,

    /// A payload for this action.
    pub payload: Option<DynActionPayload>,

    /// Reports the action's outcome, see [`CrudActionCompletion`].
    pub and_then: CrudActionCompletion,
}

/// An application action operating on one entity, offered in the views listed in `valid_in`.
#[derive(Clone)]
pub struct CrudEntityAction {
    // TODO: Both id and name could be Cow
    /// Identifies the action in requested and executing state and as UI key. Expected to be
    /// unique among the instance's entity actions.
    pub id: &'static str,
    /// Label of the action's button.
    pub name: String,
    /// Icon shown before the name.
    pub icon: Option<icondata_core::Icon>,
    /// Meaning of the action, from which renderers derive its presentation.
    pub intent: CrudActionIntent,
    /// Kinds of views offering the action.
    pub valid_in: Vec<CrudEntityViewKind>,
    /// Performs the action. Invoked when the action executes, with the current entity.
    pub action: Callback<EntityActionInput>,
    /// The view to be shown for this action.
    /// If not provided, triggering the action executes it immediately without any specific payload.
    pub view: Option<Callback<EntityActionViewInput, AnyView>>,
}

impl Debug for CrudEntityAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self {
                id,
                name,
                icon,
                intent,
                valid_in,
                action: _,
                view: _,
            } => f
                .debug_struct("CrudEntityAction")
                .field("id", id)
                .field("name", name)
                .field("icon", icon)
                .field("intent", intent)
                .field("valid_in", valid_in)
                .finish(),
        }
    }
}

impl PartialEq for CrudEntityAction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self {
                    id: l_id,
                    name: l_name,
                    icon: l_icon,
                    intent: l_intent,
                    valid_in: l_valid_in,
                    action: _l_action,
                    view: _l_modal,
                },
                Self {
                    id: r_id,
                    name: r_name,
                    icon: r_icon,
                    intent: r_intent,
                    valid_in: r_valid_in,
                    action: _r_action,
                    view: _r_modal,
                },
            ) => {
                l_id == r_id
                    && l_name == r_name
                    && l_icon == r_icon
                    && l_intent == r_intent
                    && l_valid_in == r_valid_in
            }
        }
    }
}

/// An application action operating on the resource as a whole, offered by the list view.
#[derive(Clone)]
pub struct CrudAction {
    /// Identifies the action in requested and executing state and as UI key. Expected to be
    /// unique among the instance's resource actions.
    pub id: &'static str, // TODO: Should this be Cow?
    /// Label of the action's button.
    pub name: String,
    /// Icon shown before the name.
    pub icon: Option<icondata_core::Icon>,
    /// Meaning of the action, from which renderers derive its presentation.
    pub intent: CrudActionIntent,
    /// Performs the action. Invoked when the action executes.
    pub action: Callback<ResourceActionInput>,
    /// The view to be shown for this action, e.g. a modal asking for a payload.
    /// If not provided, triggering the action executes it immediately without any payload.
    pub view: Option<Callback<ResourceActionViewInput, AnyView>>,
}

impl Debug for CrudAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self {
                id,
                name,
                icon,
                intent,
                action: _,
                view: _,
            } => f
                .debug_struct("CrudAction")
                .field("id", id)
                .field("name", name)
                .field("icon", icon)
                .field("intent", intent)
                .finish(),
        }
    }
}

impl PartialEq for CrudAction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self {
                    id: l_id,
                    name: l_name,
                    icon: l_icon,
                    intent: l_intent,
                    action: _l_action,
                    view: _l_modal,
                },
                Self {
                    id: r_id,
                    name: r_name,
                    icon: r_icon,
                    intent: r_intent,
                    action: _r_action,
                    view: _r_modal,
                },
            ) => l_id == r_id && l_name == r_name && l_icon == r_icon && l_intent == r_intent,
        }
    }
}

/// UI effects applied after an action finished.
#[derive(Debug, Clone, Default)]
pub struct CrudActionAftermath {
    /// Notification shown to the user.
    pub notification: Option<CrudNotification>,
    /// Whether the instance reloads its data.
    pub reload_data: bool,
}

/// Visual intent of an action control.
///
/// The intent describes meaning, not color. Renderers decide how to present it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum CrudActionIntent {
    /// The main action of a context.
    Primary,
    /// A supporting action.
    #[default]
    Secondary,
    /// An action creating or confirming something.
    Success,
    /// An action requiring attention.
    Warning,
    /// A destructive action.
    Danger,
}
