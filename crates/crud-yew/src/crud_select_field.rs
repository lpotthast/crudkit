use std::rc::Rc;

use crate::{crud_select::Selection, prelude::*, crud_instance::CreateOrUpdateField};
use yew::{html::Scope, prelude::*};
use yewdux::prelude::Dispatch;

use crate::{stores, CrudMainTrait};

pub enum Msg<P: 'static + CrudMainTrait, T: CrudSelectableTrait + Clone + PartialEq> {
    ParentInstanceLinksStoreUpdated(Rc<stores::instance_links::InstanceLinksStore<P>>),
    CurrentValue(Value),
    SelectionChanged(Selection<T>),
}

#[derive(Debug, Properties, PartialEq)]
pub struct Props<P: CrudMainTrait, T: CrudSelectableTrait + Clone + PartialEq> {
    pub selectable: Vec<T>,
    /// The name of the parent instance from which the referenced id should be loaded.
    pub parent_instance: String,
    /// The field of the parent, where the value is stored.
    pub parent_field: CreateOrUpdateField<P>,
}

pub struct CrudSelectField<
    P: 'static + CrudMainTrait,
    T: 'static + CrudSelectableTrait + Clone + PartialEq,
> {
    _parent_instance_links_dispatch: Dispatch<stores::instance_links::InstanceLinksStore<P>>,

    parent: Option<Scope<CrudInstance<P>>>,
    current_field_value: Option<Value>,
    selected: Selection<T>,
}

impl<P: 'static + CrudMainTrait, T: 'static + CrudSelectableTrait + Clone + PartialEq>
    CrudSelectField<P, T>
{
    fn compute_selected(&mut self, ctx: &Context<Self>) {
        self.selected = if let Some(value) = &self.current_field_value {
            let selected_options: Vec<T> = match value {
                Value::String(_) => panic!("'String' unsupported"),
                Value::Text(_) => panic!("'Text' unsupported"),
                Value::U32(_) => panic!("'U32' unsupported"),
                Value::I32(_) => panic!("'I32' unsupported"),
                Value::F32(_) => panic!("'F32' unsupported"),
                Value::Bool(_) => panic!("'Bool' unsupported"),
                Value::ValidationStatus(_) => panic!("'ValidationStatus' unsupported"),
                Value::UtcDateTime(_) => panic!("'UtcDateTime' unsupported"),
                Value::OptionalUtcDateTime(_) => panic!("'OptionalUtcDateTime' unsupported"),
                Value::OneToOneRelation(_) => panic!("'OneToOneRelation' unsupported"),
                Value::NestedTable(_) => panic!("'NestedTable' unsupported"),
                Value::Select(value) => match value {
                    Some(value) => vec![value.as_any().downcast_ref::<T>().unwrap().clone()],
                    None => vec![],
                },
            };
            let mut s = Vec::new();
            for selectable in &ctx.props().selectable {
                for selected in &selected_options {
                    if selectable == selected {
                        s.push(selectable.clone());
                    }
                }
            }
            if s.is_empty() {
                Selection::None
            } else if s.len() == 1 {
                Selection::Single(s.get(0).unwrap().clone())
            } else {
                Selection::Multiple(s)
            }
        } else {
            //log::info!("current_field_value is not set");
            Selection::None
        };
    }
}

impl<P: 'static + CrudMainTrait, T: 'static + CrudSelectableTrait + Clone + PartialEq> Component
    for CrudSelectField<P, T>
{
    type Message = Msg<P, T>;
    type Properties = Props<P, T>;

    fn create(ctx: &Context<Self>) -> Self {
        let mut this = Self {
            _parent_instance_links_dispatch: Dispatch::subscribe(
                ctx.link().callback(Msg::ParentInstanceLinksStoreUpdated),
            ),

            parent: None,
            current_field_value: None,
            selected: Selection::None,
        };
        this.compute_selected(ctx);
        this
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
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
                //log::info!("Received current value: {:?}", value);
                self.current_field_value = Some(value);
                self.compute_selected(ctx);
                true
            }
            Msg::SelectionChanged(selected) => {
                self.selected = selected.clone();
                match &self.parent {
                    Some(link) => {
                        let value = match selected {
                            Selection::None => Value::Select(None),
                            Selection::Single(option) => Value::Select(Some(Box::new(option))),
                            Selection::Multiple(_options) => {
                                log::warn!("TODO: needs implementation...");
                                Value::Select(None)
                            }
                        };

                        link.send_message(<CrudInstance<P> as Component>::Message::SaveInput((
                            ctx.props().parent_field.clone(),
                            value,
                        )));
                    }
                    None => {
                        log::warn!(
                            "Selection changed to {:?} but parent link was not yet resolved...",
                            selected
                        );
                    }
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <CrudSelect<T>
                options={ctx.props().selectable.clone()}
                selected={self.selected.clone()}
                selection_changed={ctx.link().callback(|selected| Msg::SelectionChanged(selected))} />
        }
    }
}
