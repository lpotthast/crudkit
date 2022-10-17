use std::{marker::PhantomData, rc::Rc};

use crate::{
    crud_instance::CreateOrUpdateField, crud_select::Selection, prelude::*, CrudSelectableSource,
};
use yew::{html::Scope, prelude::*};
use yewdux::prelude::Dispatch;

use crate::{stores, CrudMainTrait};

pub enum Msg<P, S, T>
where
    P: 'static + CrudMainTrait,
    S: 'static + CrudSelectableSource<Selectable = T>,
    T: 'static + CrudSelectableTrait + Clone + PartialEq
{
    SourcesLoaded(Result<Vec<S::Selectable>, Box<dyn std::error::Error + Send + Sync + 'static>>),
    ParentInstanceLinksStoreUpdated(Rc<stores::instance_links::InstanceLinksStore<P>>),
    CurrentValue(Value),
    SelectionChanged(Selection<T>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SelectMode {
    Single,
    Multiple,
    OptionalSingle,
    OptionalMultiple,
}

#[derive(Debug, Properties, PartialEq)]
pub struct Props<P>
where
    P: 'static + CrudMainTrait,
{
    pub select_mode: SelectMode,

    /// The name of the parent instance from which the referenced id should be loaded.
    pub parent_instance: String,
    /// The field of the parent, where the value is stored.
    pub parent_field: CreateOrUpdateField<P>,
}

pub struct CrudSelectField<P, S, T>
where
    P: 'static + CrudMainTrait,
    S: 'static + CrudSelectableSource<Selectable = T>,
    T: 'static + CrudSelectableTrait + Clone + PartialEq,
{
    _parent_instance_links_dispatch: Dispatch<stores::instance_links::InstanceLinksStore<P>>,

    parent: Option<Scope<CrudInstance<P>>>,

    /// The data provider for this select field.
    source: S,
    selected: Selection<T>,

    phantom_data_s: PhantomData<S>,
}

impl<P, S, T> Component for CrudSelectField<P, S, T>
where
    P: 'static + CrudMainTrait,
    S: 'static + CrudSelectableSource<Selectable = T>,
    T: 'static + CrudSelectableTrait + Clone + PartialEq,
{
    type Message = Msg<P, S, T>;
    type Properties = Props<P>;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async move {
            Msg::SourcesLoaded(S::load().await)
        });
        Self {
            _parent_instance_links_dispatch: Dispatch::subscribe(
                ctx.link().callback(Msg::ParentInstanceLinksStoreUpdated),
            ),

            parent: None,
            source: S::new(),
            selected: Selection::None,
            phantom_data_s: PhantomData {},
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SourcesLoaded(result) => {
                log::info!("loaded results");
                self.source.set_selectable(result.expect("error loading selectables..."));
                // Selectable options are now available.
                true
            }
            Msg::ParentInstanceLinksStoreUpdated(store) => {
                self.parent = store.get(ctx.props().parent_instance.as_str());

                // Whenever our parent changes, we need to fetch the current value for this field.
                if let Some(parent) = &self.parent {
                    let link = ctx.link().clone();
                    let receiver: Box<dyn FnOnce(Value)> = Box::new(move |value| {
                        link.send_message(Msg::CurrentValue(value));
                    });
                    parent.send_message(<CrudInstance<P> as Component>::Message::GetInput((
                        ctx.props().parent_field.clone(),
                        receiver,
                    )));
                }
                false
            }
            Msg::CurrentValue(value) => {
                // TODO: Improve perf. This performs a match, every take_ function also matches...
                self.selected = match value {
                    value @ Value::Select(_) => Selection::Single(value.take_select_downcast_to()),
                    value @ Value::Multiselect(_) => Selection::Multiple(value.take_multiselect_downcast_to()),
                    value @ Value::OptionalSelect(_) => match value.take_optional_select_downcast_to() {
                        Some(value) => Selection::Single(value),
                        None => Selection::None,
                    },
                    value @ Value::OptionalMultiselect(_) => match value.take_optional_multiselect_downcast_to() {
                        Some(values) => Selection::Multiple(values),
                        None => Selection::None,
                    },
                    other => panic!("Expected a select variant but got `{other:?}`."),
                };
                true
            }
            Msg::SelectionChanged(selection) => {
                self.selected = selection.clone();
                match &self.parent {
                    Some(link) => {
                        let value = match &ctx.props().select_mode {
                            SelectMode::Single => match selection {
                                Selection::None => panic!("Cannot handle Selection::None in SelectMode::Single!"),
                                Selection::Single(option) => Value::Select(Box::new(option)),
                                Selection::Multiple(_) => panic!("Cannot handle Selection::Multiple in SelectMode::Single")
                            },
                            SelectMode::Multiple => match selection {
                                Selection::None => panic!("Cannot handle Selection::None in SelectMode::Multiple"),
                                Selection::Single(_) => panic!("Cannot handle Selection::Single in SelectMode::Multiple"),
                                Selection::Multiple(options) => {
                                    let mut v: Vec<Box<dyn CrudSelectableTrait>> = Vec::new();
                                    for option in options {
                                        v.push(Box::new(option));
                                    }
                                    Value::Multiselect(v)
                                }
                            },
                            SelectMode::OptionalSingle => match selection {
                                Selection::None => Value::OptionalSelect(None),
                                Selection::Single(option) => Value::OptionalSelect(Some(Box::new(option))),
                                Selection::Multiple(_) => panic!("Cannot handle Selection::Multiple in SelectMode::OptionalSingle"),
                            },
                            SelectMode::OptionalMultiple => match selection {
                                Selection::None => Value::OptionalMultiselect(None),
                                Selection::Single(_) => panic!("Cannot handle Selection::Single in SelectMode::OptionalMultiple"),
                                Selection::Multiple(options) => {
                                    let mut v: Vec<Box<dyn CrudSelectableTrait>> = Vec::new();
                                    for option in options {
                                        v.push(Box::new(option));
                                    }
                                    Value::OptionalMultiselect(Some(v))
                                }
                            },
                        };

                        link.send_message(<CrudInstance<P> as Component>::Message::SaveInput((
                            ctx.props().parent_field.clone(),
                            Ok(value),
                        )));
                    }
                    None => {
                        log::warn!("Selection changed to {selection:?} but parent link was not yet resolved...",);
                    }
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            if let Some(selectable) = self.source.get_selectable() {
                <CrudSelect<T>
                    options={selectable}
                    selected={self.selected.clone()}
                    selection_changed={ctx.link().callback(|selected| Msg::SelectionChanged(selected))} />
            } else {
                <div>{"No options loaded. Loading..."}</div>
            }
        }
    }
}
