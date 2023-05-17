use std::{fmt::Debug, sync::Arc};
use yew::{Callback, Html};
use yew_bootstrap_icons::v1_10_3::Bi;

use crate::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum States {
    Create,
    Update,
    Read,
}

pub struct ModalGeneration<T: CrudMainTrait> {
    pub cancel: Callback<()>,
    pub execute: Callback<Option<T::ActionPayload>>,
}

pub struct EntityModalGeneration<T: CrudMainTrait> {
    pub state: T::UpdateModel,
    pub cancel: Callback<()>,
    pub execute: Callback<Option<T::ActionPayload>>,
}

#[derive(Clone)]
pub enum CrudEntityAction<T: CrudMainTrait> {
    Custom {
        id: &'static str,
        name: String,
        icon: Option<Bi>,
        variant: Variant,
        valid_in: Vec<States>,
        action: Callback<(
            T::UpdateModel,
            Option<T::ActionPayload>,
            Callback<Result<CrudActionAftermath, CrudActionAftermath>>,
        )>,
        modal: Option<Box<Arc<dyn Fn(EntityModalGeneration<T>) -> Html>>>,
    },
}

impl<T: CrudMainTrait> Debug for CrudEntityAction<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom {
                id,
                name,
                icon,
                variant,
                valid_in,
                action: _,
                modal: _,
            } => f
                .debug_struct("Custom")
                .field("id", id)
                .field("name", name)
                .field("icon", icon)
                .field("variant", variant)
                .field("valid_in", valid_in)
                .finish(),
        }
    }
}

impl<T: CrudMainTrait> PartialEq for CrudEntityAction<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Custom {
                    id: l_id,
                    name: l_name,
                    icon: l_icon,
                    variant: l_variant,
                    valid_in: l_valid_in,
                    action: _l_action,
                    modal: _l_modal,
                },
                Self::Custom {
                    id: r_id,
                    name: r_name,
                    icon: r_icon,
                    variant: r_variant,
                    valid_in: r_valid_in,
                    action: _r_action,
                    modal: _r_modal,
                },
            ) => {
                l_id == r_id
                    && l_name == r_name
                    && l_icon == r_icon
                    && l_variant == r_variant
                    && l_valid_in == r_valid_in
            }
        }
    }
}

#[derive(Clone)]
pub enum CrudAction<T: CrudMainTrait> {
    Custom {
        id: &'static str,
        name: String,
        icon: Option<leptos_icons::Icon>,
        variant: Variant,
        action: Callback<(Option<T::ActionPayload>, Callback<Result<CrudActionAftermath, CrudActionAftermath>>)>,
        modal: Option<Box<Arc<dyn Fn(ModalGeneration<T>) -> Html>>>,
    },
}

impl<T: CrudMainTrait> Debug for CrudAction<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom {
                id,
                name,
                icon,
                variant,
                action: _,
                modal: _,
            } => f
                .debug_struct("Custom")
                .field("id", id)
                .field("name", name)
                .field("icon", icon)
                .field("variant", variant)
                .finish(),
        }
    }
}

impl<T: CrudMainTrait> PartialEq for CrudAction<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Custom {
                    id: l_id,
                    name: l_name,
                    icon: l_icon,
                    variant: l_variant,
                    action: _l_action,
                    modal: _l_modal,
                },
                Self::Custom {
                    id: r_id,
                    name: r_name,
                    icon: r_icon,
                    variant: r_variant,
                    action: _r_action,
                    modal: _r_modal,
                },
            ) => l_id == r_id && l_name == r_name && l_icon == r_icon && l_variant == r_variant,
        }
    }
}

pub struct CrudActionAftermath {
    pub show_toast: Option<Toast>,
    pub reload_data: bool,
}

/// Used to model entity action such as "Open edit view", ...
/// Currently not related with the CrudAction enum.
pub trait CrudActionTrait: Debug {
    fn get_name(&self) -> String;
    fn get_icon(&self) -> Option<Bi>;
    fn eq(&self, other: &dyn CrudActionTrait) -> bool;
}

// TODO: Unused. Delete this?
#[derive(PartialEq, Debug)]
pub struct ShowListViewAction {
    name: String,
    icon: Option<Bi>,
}

impl Default for ShowListViewAction {
    fn default() -> Self {
        Self {
            name: "List".to_owned(),
            icon: Some(Bi::List),
        }
    }
}

impl CrudActionTrait for ShowListViewAction {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_icon(&self) -> Option<Bi> {
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
    icon: Option<Bi>,
}

impl Default for ShowReadViewAction {
    fn default() -> Self {
        Self {
            name: "Read".to_owned(),
            icon: Some(Bi::Eye),
        }
    }
}

impl CrudActionTrait for ShowReadViewAction {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_icon(&self) -> Option<Bi> {
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
    icon: Option<Bi>,
}

impl Default for ShowEditViewAction {
    fn default() -> Self {
        Self {
            name: "Edit".to_owned(),
            icon: Some(Bi::Pencil),
        }
    }
}

impl CrudActionTrait for ShowEditViewAction {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_icon(&self) -> Option<Bi> {
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
    icon: Option<Bi>,
}

impl Default for DeleteAction {
    fn default() -> Self {
        Self {
            name: "Delete".to_owned(),
            icon: Some(Bi::Trash),
        }
    }
}

impl CrudActionTrait for DeleteAction {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_icon(&self) -> Option<Bi> {
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
