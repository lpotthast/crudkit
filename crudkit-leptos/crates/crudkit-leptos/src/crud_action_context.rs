use std::marker::PhantomData;

use crudkit_web::CrudMainTrait;
use leptonic::prelude::*;
use leptos::*;

use crate::{crud_action::CrudActionAftermath, crud_instance::CrudInstanceContext};

pub type ActionId = &'static str;

/// Context providing the necessary signals for handling user-action requests.
#[derive(Debug, Clone)]
pub struct CrudActionContext<T: CrudMainTrait + 'static> {
    phantom: PhantomData<T>,
    actions_requested: ReadSignal<Vec<&'static str>>,
    set_actions_requested: WriteSignal<Vec<&'static str>>,
    actions_executing: ReadSignal<Vec<&'static str>>,
    set_actions_executing: WriteSignal<Vec<&'static str>>,
}

// CrudActionContext must always be Copy!
impl<T: CrudMainTrait> Copy for CrudActionContext<T> {}

impl<T: CrudMainTrait + 'static> CrudActionContext<T> {
    pub fn new() -> Self {
        // Stores the actions for which execution was requested by the user.
        let (actions_requested, set_actions_requested) = create_signal(Vec::<ActionId>::new());

        // Stores the actions which are currently executing. This allows us to not let a user execute an action while it is already executing.
        let (actions_executing, set_actions_executing) = create_signal(Vec::<ActionId>::new());

        Self {
            phantom: PhantomData::<T>::default(),
            actions_requested,
            set_actions_requested,
            actions_executing,
            set_actions_executing,
        }
    }

    pub fn request_action(&self, action_id: ActionId) {
        tracing::debug!(action_id, "request_action");
        self.set_actions_requested
            .update(|actions| actions.push(action_id));
    }

    pub fn is_action_requested(&self, action_id: ActionId) -> bool {
        self.actions_requested
            .get()
            .iter()
            .any(|it| it == &action_id)
    }

    pub fn _is_action_requested_signal(&self) -> impl Fn(ActionId) -> Signal<bool> {
        let actions_requested = self.actions_requested.clone();
        move |action_id: ActionId| {
            Signal::derive(move || actions_requested.get().iter().any(|it| it == &action_id))
        }
    }

    pub fn is_action_executing(&self, action_id: ActionId) -> bool {
        self.actions_executing.get().contains(&action_id)
    }

    pub fn cancel_action(&self, action_id: ActionId) {
        tracing::debug!(action_id, "cancel_action");
        self.set_actions_requested.update(|actions| {
            let pos = actions.iter().position(|id| id == &action_id);
            if let Some(pos) = pos {
                actions.remove(pos);
            }
        });
    }

    pub fn trigger_action(
        &self,
        action_id: ActionId,
        action_payload: Option<T::ActionPayload>,
        action: Callback<(
            Option<T::ActionPayload>,
            Callback<Result<CrudActionAftermath, CrudActionAftermath>>,
        )>,
        instance_ctx: CrudInstanceContext<T>,
    ) {
        tracing::debug!(action_id, ?action_payload, "trigger_action");

        // The user accepted the request. The action is no longer requested.
        self.set_actions_requested.update(|actions| {
            let pos = actions.iter().position(|id| id == &action_id);
            if let Some(pos) = pos {
                actions.remove(pos);
            }
        });

        // We are going to execute the action and track that here.
        self.set_actions_executing
            .update(|actions| actions.push(action_id));

        let this = self.clone();
        let finish_handler = callback(move |outcome| {
            tracing::debug!(?outcome, "action finished");

            // Regardless of the outcome, the action is now finished.
            this.set_actions_executing.update(|actions| {
                let pos = actions.iter().position(|id| id == &action_id);
                if let Some(pos) = pos {
                    actions.remove(pos);
                }
            });

            // TODO: Can we take a handler function as input to this new() fn?
            // We might have to act in a way that this list view can not comprehend and therefore let the instance handle the outcome.
            instance_ctx.handle_action_outcome(outcome);
        });

        action.call((action_payload, finish_handler));
    }

    pub fn trigger_entity_action(
        &self,
        action_id: ActionId,
        entity: T::UpdateModel,
        action_payload: Option<T::ActionPayload>,
        action: Callback<(
            T::UpdateModel,
            Option<T::ActionPayload>,
            Callback<Result<CrudActionAftermath, CrudActionAftermath>>,
        )>,
        instance_ctx: CrudInstanceContext<T>,
    ) {
        tracing::debug!(action_id, ?action_payload, "trigger_action");

        // The user accepted the request. The action is no longer requested.
        self.set_actions_requested.update(|actions| {
            let pos = actions.iter().position(|id| id == &action_id);
            if let Some(pos) = pos {
                actions.remove(pos);
            }
        });

        // We are going to execute the action and track that here.
        self.set_actions_executing
            .update(|actions| actions.push(action_id));

        let this = self.clone();
        let finish_handler = callback(move |outcome| {
            tracing::debug!(?outcome, "action finished");

            // Regardless of the outcome, the action is now finished.
            this.set_actions_executing.update(|actions| {
                let pos = actions.iter().position(|id| id == &action_id);
                if let Some(pos) = pos {
                    actions.remove(pos);
                }
            });

            // TODO: Can we take a handler function as input to this new() fn?
            // We might have to act in a way that this list view can not comprehend and therefore let the instance handle the outcome.
            instance_ctx.handle_action_outcome(outcome);
        });

        action.call((entity, action_payload, finish_handler));
    }
}
