use std::{rc::Rc, marker::PhantomData};

use crate::{
    crud_instance::CreateOrUpdateField, crud_select::Selection, prelude::*, CrudSelectableSource,
};
use yew::{html::Scope, prelude::*};
use yewdux::prelude::Dispatch;

use crate::{stores, CrudMainTrait};

pub enum Msg<P: 'static + CrudMainTrait, T: CrudSelectableTrait + Clone + PartialEq> {
    SourceLoaded,
    ParentInstanceLinksStoreUpdated(Rc<stores::instance_links::InstanceLinksStore<P>>),
    CurrentValue(Value),
    SelectionChanged(Selection<T>),
}

#[derive(Debug, Properties, PartialEq)]
pub struct Props<P>
where 
    P: 'static + CrudMainTrait,
{
    /// The name of the parent instance from which the referenced id should be loaded.
    pub parent_instance: String,
    /// The field of the parent, where the value is stored.
    pub parent_field: CreateOrUpdateField<P>
}

pub struct CrudSelectField<P, S, T>
where
    P: 'static + CrudMainTrait,
    S: 'static + CrudSelectableSource<Selectable = T>,
    T: 'static + CrudSelectableTrait + Clone + PartialEq,
{
    _parent_instance_links_dispatch: Dispatch<stores::instance_links::InstanceLinksStore<P>>,

    parent: Option<Scope<CrudInstance<P>>>,
    current_field_value: Option<Value>,
    
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
    type Message = Msg<P, T>;
    type Properties = Props<P>;

    fn create(ctx: &Context<Self>) -> Self {
        let mut this = Self {
            _parent_instance_links_dispatch: Dispatch::subscribe(
                ctx.link().callback(Msg::ParentInstanceLinksStoreUpdated),
            ),

            parent: None,
            current_field_value: None,
            source: S::new(),
            selected: Selection::None,
            phantom_data_s: PhantomData {},
        };
        let link = ctx.link().clone();
        this.source.load(Box::new(move || link.send_message(Msg::SourceLoaded)));
        this
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SourceLoaded => {
                // Selectable options are now available.
                true
            },
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
                self.current_field_value = Some(value);
                self.selected = match self.current_field_value.clone() {
                    Some(value) => value.take_select_downcast_to(),
                    None => Selection::None,
                };
                true
            }
            Msg::SelectionChanged(selection) => {
                self.selected = selection.clone();
                match &self.parent {
                    Some(link) => {
                        let value = match selection {
                            Selection::None => Value::Select(Selection::None),
                            Selection::Single(option) => Value::Select(Selection::Single(Box::new(option))),
                            Selection::Multiple(options) => {
                                let mut v: Vec<Box<dyn CrudSelectableTrait>> = Vec::new();
                                for option in options {
                                    v.push(Box::new(option));
                                }
                                Value::Select(Selection::Multiple(v))
                            }
                        };

                        link.send_message(<CrudInstance<P> as Component>::Message::SaveInput((
                            ctx.props().parent_field.clone(),
                            value,
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
            if let Some(selectable) = self.source.options() {
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
