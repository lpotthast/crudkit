use crudkit_web::CrudSelectableTrait;
use dyn_clone::DynClone;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ItemsPerPage(pub u64);

impl Default for ItemsPerPage {
    fn default() -> Self {
        Self(10)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PageNr(pub u64); // TODO: NonZero type?

impl Default for PageNr {
    fn default() -> Self {
        Self::first()
    }
}

impl PageNr {
    pub fn first() -> Self {
        Self(1)
    }
}

// TODO: Should the use ViewFn?
pub trait SelectConfigTrait: Debug + DynClone + Send + Sync + 'static {
    fn render_select(
        &self,
        selected: Signal<Box<dyn CrudSelectableTrait>>,
        set_selected: Callback<Box<dyn CrudSelectableTrait>>,
    ) -> AnyView;
    fn render_optional_select(
        &self,
        selected: Signal<Option<Box<dyn CrudSelectableTrait>>>,
        set_selected: Callback<Option<Box<dyn CrudSelectableTrait>>>,
    ) -> AnyView;
}
dyn_clone::clone_trait_object!(SelectConfigTrait);

pub type DynSelectConfig = Box<dyn SelectConfigTrait>;
