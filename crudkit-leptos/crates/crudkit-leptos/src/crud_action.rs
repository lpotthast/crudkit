use std::{fmt::Debug, rc::Rc};

use leptos_icons::BsIcon;

use crate::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum States {
    Create,
    Update,
    Read,
}

pub trait Callable<A> {
    fn call_with(&self, arg: A);
}

// TODO: Derive Debug
/// A callback which...
/// - ✅ is Clone
/// - ❌ is not Copy
/// - ✅ requires no leptos context
pub struct SimpleCallback<T: 'static, R: 'static = ()>(Rc<dyn Fn(T) -> R>);

impl<T: 'static, R: 'static> SimpleCallback<T, R> {
    pub fn new<F: Fn(T) -> R + 'static>(fun: F) -> Self {
        Self(Rc::new(fun))
    }
}

impl<T: 'static, R: 'static> std::ops::Deref for SimpleCallback<T, R> {
    type Target = Rc<dyn Fn(T) -> R>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: 'static, R: 'static> Clone for SimpleCallback<T, R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: 'static, R: 'static> Callable<T> for SimpleCallback<T, R> {
    fn call_with(&self, arg: T) {
        self.0(arg);
    }
}

// TODO: Derive Debug
/// A callback which...
/// - ❌ is Clone
/// - ✅ is Copy
/// - ⚠️ requires a leptos context
pub struct Callback<T: 'static, R: 'static = ()>(leptos::StoredValue<Box<dyn Fn(T) -> R>>);

impl<T: 'static, R: 'static> Callback<T, R> {
    pub fn new<F: Fn(T) -> R + 'static>(cx: leptos::Scope, fun: F) -> Self {
        Self(leptos::store_value(cx, Box::new(fun)))
    }
}

impl<T: 'static, R: 'static> std::ops::Deref for Callback<T, R> {
    type Target = leptos::StoredValue<Box<dyn Fn(T) -> R>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: 'static, R: 'static> Clone for Callback<T, R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: 'static, R: 'static> Copy for Callback<T, R> {}

impl<T: 'static, R: 'static> Callable<T> for Callback<T, R> {
    fn call_with(&self, arg: T) {
        self.0.with_value(|cb| cb(arg));
    }
}

// TODO: Derive Debug
/// A callback which...
/// - ✅ is Clone
/// - ✅ is Copy
/// - ⚠️ requires a leptos context
pub struct CallbackRc<T: 'static, R: 'static = ()>(leptos::StoredValue<Rc<dyn Fn(T)-> R>>);

impl<T: 'static, R: 'static> CallbackRc<T, R> {
    pub fn new<F: Fn(T) -> R + 'static>(cx: leptos::Scope, fun: F) -> Self {
        Self(leptos::store_value(cx, Rc::new(fun)))
    }
}

impl<T: 'static, R: 'static> std::ops::Deref for CallbackRc<T, R> {
    type Target = leptos::StoredValue<Rc<dyn Fn(T) -> R>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: 'static, R: 'static> Clone for CallbackRc<T, R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: 'static, R: 'static> Copy for CallbackRc<T, R> {}

impl<T: 'static, R: 'static> Callable<T> for CallbackRc<T, R> {
    fn call_with(&self, arg: T) {
        self.0.with_value(|cb| cb(arg));
    }
}

// TODO: Derive Debug
/// A callback which...
/// - ✅ is Clone
/// - ✅ is Copy
/// - ✅ is Send and Sync if T is
/// - ⚠️ requires a leptos context
pub struct CallbackArc<T: 'static, R: 'static = ()>(leptos::StoredValue<std::sync::Arc<dyn Fn(T) -> R>>);

impl<T: 'static, R: 'static> CallbackArc<T, R> {
    pub fn new<F: Fn(T) -> R + 'static>(cx: leptos::Scope, fun: F) -> Self {
        Self(leptos::store_value(cx, std::sync::Arc::new(fun)))
    }
}

impl<T: 'static, R: 'static> std::ops::Deref for CallbackArc<T, R> {
    type Target = leptos::StoredValue<std::sync::Arc<dyn Fn(T) -> R>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: 'static, R: 'static> Clone for CallbackArc<T, R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: 'static, R: 'static> Copy for CallbackArc<T, R> {}

impl<T: 'static, R: 'static> Callable<T> for CallbackArc<T, R> {
    fn call_with(&self, arg: T) {
        self.0.with_value(|cb| cb(arg));
    }
}

#[derive(Clone)]
pub struct ModalGeneration<T: CrudMainTrait + 'static> {
    pub show_when: leptos::Signal<bool>,
    pub cancel: Callback<()>,
    pub execute: Callback<Option<T::ActionPayload>>,
}

#[derive(Clone)]
pub struct EntityModalGeneration<T: CrudMainTrait + 'static> {
    pub show_when: leptos::Signal<bool>,
    pub state: leptos::Signal<Option<T::UpdateModel>>,
    pub cancel: Callback<()>,
    pub execute: Callback<Option<T::ActionPayload>>,
}

#[derive(Clone)]
pub enum CrudEntityAction<T: CrudMainTrait + 'static> {
    // TODO: Both id and name could be Cow
    Custom {
        id: &'static str,
        name: String,
        icon: Option<leptos_icons::Icon>,
        button_color: leptonic::prelude::ButtonColor,
        valid_in: Vec<States>, // TODO: Use potentially non-allocating type for small const vecs
        action: Callback<(
            T::UpdateModel,
            Option<T::ActionPayload>,
            Callback<Result<CrudActionAftermath, CrudActionAftermath>>,
        )>,
        modal: Option<Callback<(leptos::Scope, EntityModalGeneration<T>), leptos::View>>,
    },
}

impl<T: CrudMainTrait> Debug for CrudEntityAction<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom {
                id,
                name,
                icon,
                button_color,
                valid_in,
                action: _,
                modal: _,
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

impl<T: CrudMainTrait> PartialEq for CrudEntityAction<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Custom {
                    id: l_id,
                    name: l_name,
                    icon: l_icon,
                    button_color: l_button_color,
                    valid_in: l_valid_in,
                    action: _l_action,
                    modal: _l_modal,
                },
                Self::Custom {
                    id: r_id,
                    name: r_name,
                    icon: r_icon,
                    button_color: r_button_color,
                    valid_in: r_valid_in,
                    action: _r_action,
                    modal: _r_modal,
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
pub enum CrudAction<T: CrudMainTrait + 'static> {
    Custom {
        id: &'static str, // TODO: Should this be Cow?
        name: String,
        icon: Option<leptos_icons::Icon>,
        button_color: leptonic::prelude::ButtonColor,
        action: Callback<(
            Option<T::ActionPayload>,
            Callback<Result<CrudActionAftermath, CrudActionAftermath>>,
        )>,
        modal: Option<Callback<(leptos::Scope, ModalGeneration<T>), leptos::View>>,
    },
}

impl<T: CrudMainTrait> Debug for CrudAction<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom {
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

impl<T: CrudMainTrait> PartialEq for CrudAction<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Custom {
                    id: l_id,
                    name: l_name,
                    icon: l_icon,
                    button_color: l_button_color,
                    action: _l_action,
                    modal: _l_modal,
                },
                Self::Custom {
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
    pub show_toast: Option<leptonic::toast::Toast>,
    pub reload_data: bool,
}

/// Used to model entity action such as "Open edit view", ...
/// Currently not related with the CrudAction enum.
pub trait CrudActionTrait: Debug {
    fn get_name(&self) -> String;
    fn get_icon(&self) -> Option<leptos_icons::Icon>;
    fn eq(&self, other: &dyn CrudActionTrait) -> bool;
}

// TODO: Unused. Delete this?
#[derive(PartialEq, Debug)]
pub struct ShowListViewAction {
    name: String,
    icon: Option<leptos_icons::Icon>,
}

impl Default for ShowListViewAction {
    fn default() -> Self {
        Self {
            name: "List".to_owned(),
            icon: Some(BsIcon::BsList.into()),
        }
    }
}

impl CrudActionTrait for ShowListViewAction {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_icon(&self) -> Option<leptos_icons::Icon> {
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
    icon: Option<leptos_icons::Icon>,
}

impl Default for ShowReadViewAction {
    fn default() -> Self {
        Self {
            name: "Read".to_owned(),
            icon: Some(BsIcon::BsEye.into()),
        }
    }
}

impl CrudActionTrait for ShowReadViewAction {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_icon(&self) -> Option<leptos_icons::Icon> {
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
    icon: Option<leptos_icons::Icon>,
}

impl Default for ShowEditViewAction {
    fn default() -> Self {
        Self {
            name: "Edit".to_owned(),
            icon: Some(BsIcon::BsPencil.into()),
        }
    }
}

impl CrudActionTrait for ShowEditViewAction {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_icon(&self) -> Option<leptos_icons::Icon> {
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
    icon: Option<leptos_icons::Icon>,
}

impl Default for DeleteAction {
    fn default() -> Self {
        Self {
            name: "Delete".to_owned(),
            icon: Some(BsIcon::BsTrash.into()),
        }
    }
}

impl CrudActionTrait for DeleteAction {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_icon(&self) -> Option<leptos_icons::Icon> {
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
