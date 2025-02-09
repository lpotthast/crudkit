use crudkit_web::CrudSelectableTrait;
use dyn_clone::DynClone;
use leptos::prelude::*;
use std::fmt::Debug;

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
