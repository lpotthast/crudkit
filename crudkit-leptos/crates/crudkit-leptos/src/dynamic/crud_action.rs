use crate::prelude::*;
use crudkit_web::dynamic::prelude::*;
use leptonic::prelude::icondata;
use leptos::prelude::*;
use std::{fmt::Debug, rc::Rc};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum States {
    Create,
    Update,
    Read,
}

#[derive(Clone)]
pub struct ResourceActionViewInput {
    pub show_when: Signal<bool>,
    pub cancel: Callback<()>,
    pub execute: Callback<Option<AnyActionPayload>>,
}

#[derive(Clone, Copy)]
pub struct EntityActionViewInput {
    pub show_when: Signal<bool>,
    pub state: Signal<Option<AnyModel>>,
    pub cancel: Callback<()>,
    pub execute: Callback<Option<AnyActionPayload>>,
}

pub struct ResourceActionInput {
    pub payload: Option<AnyActionPayload>,
    pub after: Callback<Result<CrudActionAftermath, CrudActionAftermath>>,
}

/// The concrete data to perform an entity-action with.
pub struct EntityActionInput {
    pub update_model: AnyModel,
    pub payload: Option<AnyActionPayload>,
    pub after: Callback<Result<CrudActionAftermath, CrudActionAftermath>>,
}

#[derive(Clone)]
pub struct CrudEntityAction {
    // TODO: Both id and name could be Cow
    pub id: &'static str,
    pub name: String,
    pub icon: Option<icondata::Icon>,
    pub button_color: leptonic::components::prelude::ButtonColor,
    pub valid_in: Vec<States>,
    pub action: Callback<EntityActionInput>,
    /// The view to be shown for this action.
    /// If not provided, triggering the action executes it immediately.
    pub view: Option<Callback<EntityActionViewInput, AnyView>>,
}

impl Debug for CrudEntityAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self {
                id,
                name,
                icon,
                button_color,
                valid_in,
                action: _,
                view: _,
            } => f
                .debug_struct("Custom")
                .field("id", id)
                .field("name", name)
                .field("icon", icon)
                .field("button_color", button_color)
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
                    button_color: l_button_color,
                    valid_in: l_valid_in,
                    action: _l_action,
                    view: _l_modal,
                },
                Self {
                    id: r_id,
                    name: r_name,
                    icon: r_icon,
                    button_color: r_button_color,
                    valid_in: r_valid_in,
                    action: _r_action,
                    view: _r_modal,
                },
            ) => {
                l_id == r_id
                    && l_name == r_name
                    && l_icon == r_icon
                    && l_button_color == r_button_color
                    && l_valid_in == r_valid_in
            }
        }
    }
}

#[derive(Clone)]
pub struct ActionModalGen<T, R>(pub Rc<Box<dyn Fn(T) -> R>>);

impl<T, R> ActionModalGen<T, R> {
    pub fn of<F: Fn(T) -> R + 'static>(fun: F) -> Self {
        Self(Rc::new(Box::new(fun)))
    }
}

#[derive(Clone)]
pub struct CrudAction {
    pub id: &'static str, // TODO: Should this be Cow?
    pub name: String,
    pub icon: Option<icondata::Icon>,
    pub button_color: leptonic::components::prelude::ButtonColor,
    pub action: Callback<ResourceActionInput>,
    pub modal: Option<Callback<ResourceActionViewInput, AnyView>>,
}

impl Debug for CrudAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self {
                id,
                name,
                icon,
                button_color,
                action: _,
                modal: _,
            } => f
                .debug_struct("Custom")
                .field("id", id)
                .field("name", name)
                .field("icon", icon)
                .field("button_color", button_color)
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
                    button_color: l_button_color,
                    action: _l_action,
                    modal: _l_modal,
                },
                Self {
                    id: r_id,
                    name: r_name,
                    icon: r_icon,
                    button_color: r_button_color,
                    action: _r_action,
                    modal: _r_modal,
                },
            ) => {
                l_id == r_id
                    && l_name == r_name
                    && l_icon == r_icon
                    && l_button_color == r_button_color
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CrudActionAftermath {
    pub show_toast: Option<leptonic::components::prelude::Toast>,
    pub reload_data: bool,
}

/// Used to model entity action such as "Open edit view", ...
/// Currently not related with the CrudAction enum.
///
/// TODO: Should this be dyn compatible?
/// TODO: Move methods to AnyAction!
pub trait CrudActionTrait: Debug + Send + Sync {
    fn get_name(&self) -> String;
    fn get_icon(&self) -> Option<icondata::Icon>;
    fn eq(&self, other: &dyn CrudActionTrait) -> bool;
}

// TODO: Unused. Delete this?
#[derive(PartialEq, Debug)]
pub struct ShowListViewAction {
    name: String,
    icon: Option<icondata::Icon>,
}

impl Default for ShowListViewAction {
    fn default() -> Self {
        Self {
            name: "List".to_owned(),
            icon: Some(icondata::BsList),
        }
    }
}

impl CrudActionTrait for ShowListViewAction {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_icon(&self) -> Option<icondata::Icon> {
        self.icon
    }

    fn eq(&self, other: &dyn CrudActionTrait) -> bool {
        self.get_icon() == other.get_icon() && self.get_name() == other.get_name()
    }
}

// TODO: Unused. Delete this?
#[derive(PartialEq, Debug)]
pub struct ShowReadViewAction {
    name: String,
    icon: Option<icondata::Icon>,
}

impl Default for ShowReadViewAction {
    fn default() -> Self {
        Self {
            name: "Read".to_owned(),
            icon: Some(icondata::BsEye),
        }
    }
}

impl CrudActionTrait for ShowReadViewAction {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_icon(&self) -> Option<icondata::Icon> {
        self.icon
    }

    fn eq(&self, other: &dyn CrudActionTrait) -> bool {
        self.get_icon() == other.get_icon() && self.get_name() == other.get_name()
    }
}

// TODO: Unused. Delete this?
#[derive(PartialEq, Debug)]
pub struct ShowEditViewAction {
    name: String,
    icon: Option<icondata::Icon>,
}

impl Default for ShowEditViewAction {
    fn default() -> Self {
        Self {
            name: "Edit".to_owned(),
            icon: Some(icondata::BsPencil),
        }
    }
}

impl CrudActionTrait for ShowEditViewAction {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_icon(&self) -> Option<icondata::Icon> {
        self.icon
    }

    fn eq(&self, other: &dyn CrudActionTrait) -> bool {
        self.get_icon() == other.get_icon() && self.get_name() == other.get_name()
    }
}

// TODO: Unused. Delete this?
#[derive(PartialEq, Debug)]
pub struct DeleteAction {
    name: String,
    icon: Option<icondata::Icon>,
}

impl Default for DeleteAction {
    fn default() -> Self {
        Self {
            name: "Delete".to_owned(),
            icon: Some(icondata::BsTrash),
        }
    }
}

impl CrudActionTrait for DeleteAction {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_icon(&self) -> Option<icondata::Icon> {
        self.icon
    }

    fn eq(&self, other: &dyn CrudActionTrait) -> bool {
        self.get_icon() == other.get_icon() && self.get_name() == other.get_name()
    }
}

impl PartialEq for dyn CrudActionTrait + '_ {
    fn eq(&self, that: &dyn CrudActionTrait) -> bool {
        CrudActionTrait::eq(self, that)
    }
}

impl PartialEq<dyn CrudActionTrait> for Box<dyn CrudActionTrait + '_> {
    fn eq(&self, that: &dyn CrudActionTrait) -> bool {
        CrudActionTrait::eq(&**self, that)
    }
}
