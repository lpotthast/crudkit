use std::rc::Rc;

use crate::{crud_select::Selection, prelude::*};
use crud_shared_types::{Condition, ConditionClause, ConditionClauseValue, ConditionElement};
use yew::{html::Scope, prelude::*};
use yewdux::prelude::Dispatch;

use crate::{
    crud_instance::NestedConfig,
    services::crud_rest_data_provider::{CrudRestDataProvider, ReadMany},
    stores,
    types::RequestError,
    CrudDataTrait,
};

pub enum Msg<P: 'static + CrudDataTrait, T: CrudDataTrait> {
    ParentInstanceLinksStoreUpdated(Rc<stores::instance_links::InstanceLinksStore<P>>),
    InstanceViewsStoreUpdated(Rc<stores::instance_views::InstanceViewsStore>),
    CurrentValue(Value),
    LoadedData(Result<Vec<T>, RequestError>),
    SelectionChanged(Selection<T>),
}

#[derive(Properties, PartialEq)]
pub struct Props<P: CrudDataTrait> {
    pub api_base_url: String,
    pub nested: NestedConfig,
    pub field: P::FieldType,
}

pub struct CrudRelatedField<P: 'static + CrudDataTrait, T: 'static + CrudDataTrait> {
    parent_instance_links_store: Rc<stores::instance_links::InstanceLinksStore<P>>,
    parent_instance_links_dispatch: Dispatch<stores::instance_links::InstanceLinksStore<P>>,
    instance_views_store: Rc<stores::instance_views::InstanceViewsStore>,
    instance_views_dispatch: Dispatch<stores::instance_views::InstanceViewsStore>,

    parent: Option<Scope<CrudInstance<P>>>,
    current_field_value: Option<Value>,
    data_provider: CrudRestDataProvider<T>,
    data: Option<Result<Vec<T>, RequestError>>,
    selected: Selection<T>,
}

impl<P: 'static + CrudDataTrait, T: 'static + CrudDataTrait> CrudRelatedField<P, T> {
    fn compute_selected(&mut self) {
        self.selected = if let Some(value) = &self.current_field_value {
            let selected_ids = match value {
                Value::String(_) => panic!("unsupported"),
                Value::Text(_) => panic!("unsupported"),
                Value::U32(u32) => vec![*u32],
                Value::Bool(_) => panic!("unsupported"),
                Value::UtcDateTime(_) => panic!("unsupported"),
                Value::OneToOneRelation(some_u32) => match some_u32 {
                    Some(u32) => vec![*u32],
                    None => vec![],
                },
                Value::NestedTable(_) => panic!("unsupported"),
            };
            if let Some(data) = &self.data {
                match data {
                    Ok(data) => {
                        let mut s = Vec::new();
                        for entity in data {
                            let ent_id = value_as_u32(&T::get_id_field().get_value(entity)).unwrap();
                            for selected_id in &selected_ids {
                                log::info!("comparing {ent_id} - {selected_id}");
                                if selected_id == &ent_id {
                                    s.push(entity.clone());
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
                    },
                    Err(_) => Selection::None,
                }
            } else {
                log::info!("data not yet loaded");
                Selection::None
            }
        } else {
            log::info!("current_field_value is not set");
            Selection::None
        };
    }
}

fn value_as_u32(value: &Value) -> Option<u32> {
    match value {
        Value::String(_) => panic!("unsupported"),
        Value::Text(_) => panic!("unsupported"),
        Value::U32(u32) => Some(*u32),
        Value::Bool(_) => panic!("unsupported"),
        Value::UtcDateTime(_) => panic!("unsupported"),
        Value::OneToOneRelation(some_u32) => match some_u32 {
            Some(u32) => Some(*u32),
            None => None,
        },
        Value::NestedTable(_) => panic!("unsupported"),
    }
}

/*
TODO:
- receive parent id, look at parent CrudView
- Only displayable if parent has an id. Relation can not be resolved otherwise
- Load list of relatable entries
- Display based on current view
- Pass data events through global store...

*/

impl<P: 'static + CrudDataTrait, T: 'static + CrudDataTrait> Component for CrudRelatedField<P, T> {
    type Message = Msg<P, T>;
    type Properties = Props<P>;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            parent_instance_links_dispatch: Dispatch::subscribe(
                ctx.link().callback(Msg::ParentInstanceLinksStoreUpdated),
            ),
            parent_instance_links_store: Default::default(),
            instance_views_dispatch: Dispatch::subscribe(
                ctx.link().callback(Msg::InstanceViewsStoreUpdated),
            ),
            instance_views_store: Default::default(),

            parent: None,
            current_field_value: None,
            data_provider: CrudRestDataProvider::new(ctx.props().api_base_url.clone()),
            data: None,
            selected: Selection::None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ParentInstanceLinksStoreUpdated(store) => {
                self.parent = store.get(ctx.props().nested.parent_instance.as_str());
                
                // Whenever out parent changes, we need to fetch the current value for this field.
                if let Some(parent) = &self.parent {
                    let link = ctx.link().clone();
                    let receiver: Box<dyn FnOnce(Value)> = Box::new(move |value| {
                        link.send_message(Msg::CurrentValue(value));
                    });
                    parent.send_message(<CrudInstance<P> as Component>::Message::GetInput((ctx.props().field.clone(), receiver)));
                }
                false
            }
            Msg::InstanceViewsStoreUpdated(store) => {
                // TODO: Do we really need to store this?
                self.instance_views_store = store;
                match self
                    .instance_views_store
                    .get(ctx.props().nested.parent_instance.as_str())
                {
                    Some(parent_view) => match parent_view {
                        crate::CrudView::List | crate::CrudView::Create => {
                            log::warn!("Cannot show this field in List or Create view...");
                        }
                        crate::CrudView::Read(id) | crate::CrudView::Edit(id) => {
                            let mut data_provider =
                                CrudRestDataProvider::new(ctx.props().api_base_url.clone());
                            data_provider.set_base_condition(Some(Condition::All(vec![
                                ConditionElement::Clause(ConditionClause {
                                    column_name: ctx.props().nested.reference_field.clone(),
                                    operator: crud_shared_types::Operator::Equal,
                                    value: ConditionClauseValue::U32(id),
                                }),
                            ])));

                            ctx.link().send_future(async move {
                                Msg::LoadedData(
                                    data_provider
                                        .read_many(ReadMany {
                                            limit: None,
                                            skip: None,
                                            order_by: None,
                                            condition: None,
                                        })
                                        .await,
                                )
                            });
                        }
                    },
                    None => todo!(),
                }
                false
            }
            Msg::CurrentValue(value) => {
                log::info!("Received current value: {:?}", value);
                self.current_field_value = Some(value);
                self.compute_selected();
                true
            }
            Msg::LoadedData(result) => {
                self.data = Some(result);
                self.compute_selected();
                true
            }
            Msg::SelectionChanged(selected) => {
                self.selected = selected.clone();
                match &self.parent {
                    Some(link) => {
                        let value = match selected {
                            Selection::None => Value::OneToOneRelation(None),
                            Selection::Single(entity) => {
                                T::get_field(ctx.props().nested.parent_field.as_str())
                                    .get_value(&entity)
                            }
                            Selection::Multiple(_entities) => {
                                log::warn!("TODO: needs implementation...");
                                Value::OneToOneRelation(None)
                            }
                        };

                        link.send_message(<CrudInstance<P> as Component>::Message::SaveInput((
                            ctx.props().field.clone(),
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
            match &self.data {
                Some(result) => match result {
                    Ok(data) => html! {
                        <CrudSelect<T>
                            options={data.clone()}
                            selected={self.selected.clone()}
                            selection_changed={ctx.link().callback(|selected| Msg::SelectionChanged(selected))} />
                    },
                    Err(err) => html! { format!("Error: {}", err.to_string()) },
                },
                None => html! { "Loading..." },
            }
        }
    }
}
