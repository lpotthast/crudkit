use std::rc::Rc;

use crudkit_condition::{Condition, ConditionClause, ConditionElement};
use tracing::warn;
use yew::html::prelude::*;
use yewdux::prelude::Dispatch;

use crate::{crud_instance::CreateOrUpdateField, crud_select::Selection, prelude::*, stores};

pub enum Msg<P: 'static + CrudMainTrait, T: CrudMainTrait> {
    ParentInstanceLinksStoreUpdated(Rc<stores::instance_links::InstanceLinksStore<P>>),
    InstanceViewsStoreUpdated(Rc<stores::instance_views::InstanceViewsStore>),
    CurrentValue(Value),
    LoadedData(Result<Vec<T::ReadModel>, RequestError>),
    SelectionChanged(Selection<T::ReadModel>),
}

/// P: The parent entity
/// T: The own entity
#[derive(Properties, PartialEq)]
pub struct Props<P: CrudMainTrait, T: CrudMainTrait> {
    pub api_base_url: String,
    /// The name of the parent instance from which the referenced id should be loaded.
    pub parent_instance: String,
    /// The field of the parent, where another entry is referenced.
    pub parent_field: CreateOrUpdateField<P>,
    /// The field of the related entry whose value is stored in the parent.
    pub connect_field: <T::ReadModel as CrudDataTrait>::Field,
    /// The field in which the reference to the parent is store.
    pub parent_reverse_field: <T::ReadModel as CrudDataTrait>::Field,
}

pub struct CrudRelatedField<P: 'static + CrudMainTrait, T: 'static + CrudMainTrait> {
    _parent_instance_links_dispatch: Dispatch<stores::instance_links::InstanceLinksStore<P>>,
    _instance_views_dispatch: Dispatch<stores::instance_views::InstanceViewsStore>,

    parent: Option<CrudInstance<P>>,
    current_field_value: Option<Value>,
    data: Option<Result<Vec<T::ReadModel>, RequestError>>,
    selected: Selection<T::ReadModel>,
}

impl<P: 'static + CrudMainTrait, T: 'static + CrudMainTrait> CrudRelatedField<P, T> {
    fn compute_selected(&mut self, ctx: &Context<Self>) {
        self.selected = if let Some(value) = &self.current_field_value {
            // TODO: Extract different types of ids: u32, i32, uuid, etc...
            let selected_ids = value_as_u32_vec(value);
            if let Some(data) = &self.data {
                match data {
                    Ok(data) => {
                        let mut s = Vec::new();
                        for entity in data {
                            let ent_id =
                                value_as_u32(&ctx.props().connect_field.get_value(entity)).unwrap();
                            for selected_id in &selected_ids {
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
                    }
                    Err(_) => Selection::None,
                }
            } else {
                //info!("data not yet loaded");
                Selection::None
            }
        } else {
            //info!("current_field_value is not set");
            Selection::None
        };
    }
}

fn value_as_u32_vec(value: &Value) -> Vec<u32> {
    match value {
        Value::String(_) => panic!("'String' unsupported"),
        Value::Text(_) => panic!("'Text' unsupported"),
        Value::Json(_) => panic!("'Json' unsupported"),
        Value::OptionalJson(_) => panic!("'OptionalJson' unsupported"),
        Value::UuidV4(_) => panic!("'UuidV4' unsupported"),
        Value::UuidV7(_) => panic!("'UuidV7' unsupported"),
        Value::U32(u32) => vec![*u32],
        Value::OptionalU32(optional_u32) => match optional_u32 {
            Some(u32) => vec![*u32],
            None => vec![],
        },
        Value::I32(_) => panic!("'I32' unsupported"),
        Value::I64(_) => panic!("'I64' unsupported"),
        Value::OptionalI32(_) => panic!("'OptionalI32' unsupported"),
        Value::OptionalI64(_) => panic!("'OptionalI64' unsupported"),
        Value::F32(_) => panic!("'F32' unsupported"),
        Value::Bool(_) => panic!("'Bool' unsupported"),
        Value::ValidationStatus(_) => panic!("'ValidationStatus' unsupported"),
        Value::PrimitiveDateTime(_) => panic!("PrimitiveDateTime' unsupported"),
        Value::OffsetDateTime(_) => panic!("OffsetDateTime' unsupported"),
        Value::OptionalPrimitiveDateTime(_) => panic!("'OptionalPrimitiveDateTime' unsupported"),
        Value::OptionalOffsetDateTime(_) => panic!("'OptionalOffsetDateTime' unsupported"),
        Value::OneToOneRelation(some_u32) => match some_u32 {
            Some(u32) => vec![*u32],
            None => vec![],
        },
        Value::NestedTable(_) => panic!("'NestedTable' unsupported"),
        Value::Custom(_) => panic!("'Custom' unsupported"),
        Value::Select(_) => panic!("'Select' unsupported"),
        Value::Multiselect(_) => panic!("'Multiselect' unsupported"),
        Value::OptionalSelect(_) => panic!("'OptionalSelect' unsupported"),
        Value::OptionalMultiselect(_) => panic!("'OptionalMultiselect' unsupported"),
    }
}

fn value_as_u32(value: &Value) -> Option<u32> {
    match value {
        Value::String(_) => panic!("'String' unsupported"),
        Value::Text(_) => panic!("'Text' unsupported"),
        Value::Json(_) => panic!("'Json' unsupported"),
        Value::OptionalJson(_) => panic!("'OptionalJson' unsupported"),
        Value::UuidV4(_) => panic!("'UuidV4' unsupported"),
        Value::UuidV7(_) => panic!("'UuidV7' unsupported"),
        Value::U32(u32) => Some(*u32),
        Value::OptionalU32(optional_u32) => match optional_u32 {
            Some(u32) => Some(*u32),
            None => None,
        },
        Value::I32(_) => panic!("'I32' unsupported"),
        Value::I64(_) => panic!("'I64' unsupported"),
        Value::OptionalI32(_) => panic!("'OptionalI32' unsupported"),
        Value::OptionalI64(_) => panic!("'OptionalI64' unsupported"),
        Value::F32(_) => panic!("'F32' unsupported"),
        Value::Bool(_) => panic!("'Bool' unsupported"),
        Value::ValidationStatus(_) => panic!("'ValidationStatus' unsupported"),
        Value::PrimitiveDateTime(_) => panic!("PrimitiveDateTime' unsupported"),
        Value::OffsetDateTime(_) => panic!("OffsetDateTime' unsupported"),
        Value::OptionalPrimitiveDateTime(_) => panic!("'OptionalPrimitiveDateTime' unsupported"),
        Value::OptionalOffsetDateTime(_) => panic!("'OptionalOffsetDateTime' unsupported"),
        Value::OneToOneRelation(some_u32) => match some_u32 {
            Some(u32) => Some(*u32),
            None => None,
        },
        Value::NestedTable(_) => panic!("'NestedTable' unsupported"),
        Value::Custom(_) => panic!("'Custom' unsupported"),
        Value::Select(_) => panic!("'Select' unsupported"),
        Value::Multiselect(_) => panic!("'Multiselect' unsupported"),
        Value::OptionalSelect(_) => panic!("'OptionalSelect' unsupported"),
        Value::OptionalMultiselect(_) => panic!("'OptionalMultiselect' unsupported"),
    }
}

/*
TODO:
- receive parent id, look at parent CrudView
- Only displayable if parent has an id. Relation can not be resolved otherwise
- Load list of relatable entries
- Display based on current view

*/

impl<P: 'static + CrudMainTrait, T: 'static + CrudMainTrait> Component for CrudRelatedField<P, T> {
    type Message = Msg<P, T>;
    type Properties = Props<P, T>;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            _parent_instance_links_dispatch: Dispatch::subscribe(
                ctx.link().callback(Msg::ParentInstanceLinksStoreUpdated),
            ),
            _instance_views_dispatch: Dispatch::subscribe(
                ctx.link().callback(Msg::InstanceViewsStoreUpdated),
            ),

            parent: None,
            current_field_value: None,
            data: None,
            selected: Selection::None,
        }
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
            Msg::InstanceViewsStoreUpdated(store) => {
                // TODO: Do we really need to store this?
                match store.get(ctx.props().parent_instance.as_str()) {
                    Some(parent_view) => match parent_view {
                        SerializableCrudView::List | SerializableCrudView::Create => {
                            warn!("Cannot show this field in List or Create view...");
                        }
                        SerializableCrudView::Read(id) | SerializableCrudView::Edit(id) => {
                            let mut data_provider: CrudRestDataProvider<T> =
                                CrudRestDataProvider::new(ctx.props().api_base_url.clone());

                            let (_field_name, value) =
                                id.0.iter()
                                    .find(|(field_name, _value)| {
                                        field_name == ctx.props().parent_reverse_field.get_name()
                                    })
                                    .expect("related parent field must be part of the parents id!");

                            data_provider.set_base_condition(Some(Condition::All(vec![
                                ConditionElement::Clause(ConditionClause {
                                    column_name: ctx
                                        .props()
                                        .parent_reverse_field
                                        .get_name()
                                        .to_owned(),
                                    operator: crudkit_condition::Operator::Equal,
                                    value: value.clone().into(),
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
                //info!("Received current value: {:?}", value);
                self.current_field_value = Some(value);
                self.compute_selected(ctx);
                true
            }
            Msg::LoadedData(result) => {
                self.data = Some(result);
                self.compute_selected(ctx);
                true
            }
            Msg::SelectionChanged(selected) => {
                self.selected = selected.clone();
                match &self.parent {
                    Some(link) => {
                        let value = match selected {
                            Selection::None => Value::OneToOneRelation(None),
                            Selection::Single(entity) => {
                                ctx.props().connect_field.get_value(&entity)
                            }
                            Selection::Multiple(_entities) => {
                                warn!("TODO: needs implementation...");
                                Value::OneToOneRelation(None)
                            }
                        };

                        link.send_message(<CrudInstance<P> as Component>::Message::SaveInput((
                            ctx.props().parent_field.clone(),
                            Ok(value),
                        )));
                    }
                    None => {
                        warn!(
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
                        <CrudSelect<T::ReadModel>
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
