use std::fmt::Debug;
use yew::Callback;
use yewbi::Bi;

use crate::{Variant, types::toasts::Toast};

#[derive(Clone)]
pub enum CrudAction {
    Custom {
        id: &'static str,
        name: String,
        icon: Option<Bi>,
        variant: Variant,
        action: Callback<Callback<Result<CrudActionAftermath, CrudActionAftermath>>>,
    },
}

impl Debug for CrudAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom {
                id,
                name,
                icon,
                variant,
                action: _,
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

impl PartialEq for CrudAction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Custom {
                    id: l_id,
                    name: l_name,
                    icon: l_icon,
                    variant: l_variant,
                    action: _l_action,
                },
                Self::Custom {
                    id: r_id,
                    name: r_name,
                    icon: r_icon,
                    variant: r_variant,
                    action: _r_action,
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
