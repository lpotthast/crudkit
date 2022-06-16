use crud_shared_types::Order;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use yew::{html::ChildrenRenderer, prelude::*, virtual_dom::VChild};
use yewdux::prelude::*;

use crate::DateTimeDisplay;

use super::{
    prelude::*,
    services::controller::{delete_by_id, DeleteById},
    stores,
    types::RequestError,
};

pub enum Msg<T: CrudDataTrait> {
    InstanceConfigStoreUpdated(Rc<stores::instance::InstanceStore<T>>),
    InstanceViewsStoreUpdated(Rc<stores::instance_views::InstanceViewsStore>),
    List,
    Create,
    EntityCreated((T, Option<CrudView>)),
    Read(T),
    Edit(T),
    Delete(T),
    DeleteCanceled,
    DeleteApproved,
    Deleted(Result<Option<i32>, RequestError>),
    OrderBy((T::FieldType, OrderByUpdateOptions)),
    PageSelected(u64),
    Action((Rc<Box<dyn CrudActionTrait>>, T)),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub struct NestedConfig {
    /// The name of the parent instance from which the referenced id should be loaded.
    pub parent_instance: String,

    /// The field of the parent instance from which the referenced id should be loaded.
    pub parent_field: String,

    /// The `own` field in which the reference is stored.
    pub reference_field: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrudInstanceConfig<T: CrudDataTrait> {
    pub view: CrudView,
    pub headers: Vec<(T::FieldType, HeaderOptions)>,
    // serde bound used as described in: https://github.com/serde-rs/serde/issues/1296
    #[serde(bound = "")]
    pub elements: Vec<Elem<T>>,
    pub order_by: IndexMap<T::FieldType, Order>,
    pub items_per_page: u64,
    pub page: u64,
    pub nested: Option<NestedConfig>,
}

impl<T: CrudDataTrait> Default for CrudInstanceConfig<T> {
    fn default() -> Self {
        let mut order_by = IndexMap::default();
        order_by.insert(T::get_id_field(), Order::Asc);
        Self {
            view: CrudView::default(),
            headers: vec![(
                T::get_id_field(),
                HeaderOptions {
                    display_name: "Id".to_owned(),
                    ordering_allowed: true,
                    date_time_display: DateTimeDisplay::LocalizedLocal,
                },
            )],
            elements: vec![],
            order_by,
            items_per_page: 10,
            page: 1,
            nested: None,
        }
    }
}

impl<T: CrudDataTrait> CrudInstanceConfig<T> {
    fn update_order_by(&mut self, field: T::FieldType, options: OrderByUpdateOptions) {
        let prev = self.order_by.get(&field).cloned();
        if !options.append {
            self.order_by.clear();
        }
        self.order_by.insert(
            field,
            match prev {
                Some(order) => match order {
                    Order::Asc => Order::Desc,
                    Order::Desc => Order::Asc,
                },
                None => Order::Asc,
            },
        );
    }
}

#[derive(Clone, derive_more::From, PartialEq)]
pub enum Item {
    NestedInstance(VChild<CrudNestedInstance>),
}

// Now, we implement `Into<Html>` so that yew knows how to render `Item`.
#[allow(clippy::from_over_into)]
impl Into<Html> for Item {
    fn into(self) -> Html {
        match self {
            Item::NestedInstance(child) => child.into(),
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct Props<T: CrudDataTrait> {
    #[prop_or_default]
    pub children: ChildrenRenderer<Item>,
    pub name: String,
    pub base_url: String,
    pub config: CrudInstanceConfig<T>,
    pub portal_target: Option<String>,
}

pub struct CrudInstance<T: 'static + CrudDataTrait> {
    instance_store: Rc<stores::instance::InstanceStore<T>>,
    instance_dispatch: Dispatch<stores::instance::InstanceStore<T>>,
    instance_views_store: Rc<stores::instance_views::InstanceViewsStore>,
    instance_views_dispatch: Dispatch<stores::instance_views::InstanceViewsStore>,
    config: CrudInstanceConfig<T>,
    entity_to_delete: Option<T>,
}

impl<T: 'static + CrudDataTrait> CrudInstance<T> {
    fn store_config(&self, ctx: &Context<CrudInstance<T>>) {
        let name = ctx.props().name.clone();
        let config = self.config.clone();
        self.instance_dispatch
            .reduce(|state| state.save(name, config));

        let name = ctx.props().name.clone();
        let view = self.config.view.clone();
        self.instance_views_dispatch
            .reduce(|state| state.save(name, view));
    }

    fn set_view(&mut self, view: CrudView) {
        self.config.view = view;
    }

    fn render(&self, ctx: &Context<CrudInstance<T>>) -> Html {
        html! {
            <div class={"crud-instance"}>
                <div class={"body"}>
                    {
                        match self.config.view {
                            CrudView::List => {
                                html! {
                                    <CrudListView<T>
                                        api_base_url={ctx.props().base_url.clone()}
                                        children={ctx.props().children.clone()}
                                        config={self.config.clone()}
                                        on_create={ctx.link().callback(|_| Msg::Create)}
                                        on_read={ctx.link().callback(Msg::Read)}
                                        on_edit={ctx.link().callback(Msg::Edit)}
                                        on_delete={ctx.link().callback(Msg::Delete)}
                                        on_order_by={ctx.link().callback(Msg::OrderBy)}
                                        on_page_selected={ctx.link().callback(Msg::PageSelected)}
                                        on_action={ctx.link().callback(Msg::Action)}
                                    />
                                }
                            },
                            CrudView::Create => {
                                html! {
                                    <CrudCreateView<T>
                                        api_base_url={ctx.props().base_url.clone()}
                                        parent_id={if let Some(nested) = &ctx.props().config.nested {
                                            match self.instance_views_store.get(nested.parent_instance.as_str()) {
                                                Some(parent_view) => match parent_view {
                                                    CrudView::List => None,
                                                    CrudView::Create => None,
                                                    CrudView::Read(id) => Some(id),
                                                    CrudView::Edit(id) => Some(id),
                                                },
                                                None => {
                                                    log::info!("no parent config");
                                                    None
                                                },
                                            }
                                        } else {
                                            log::info!("not nested");
                                            None
                                        }}
                                        children={ctx.props().children.clone()}
                                        config={self.config.clone()}
                                        list_view_available={true}
                                        on_list_view={ctx.link().callback(|_| Msg::List)}
                                        on_entity_created={ctx.link().callback(Msg::EntityCreated)}
                                    />
                                }
                            },
                            CrudView::Read(id) => {
                                html! {
                                    <CrudReadView<T>
                                        api_base_url={ctx.props().base_url.clone()}
                                        children={ctx.props().children.clone()}
                                        config={self.config.clone()}
                                        id={id}
                                        list_view_available={true}
                                        on_list_view={ctx.link().callback(|_| Msg::List)}
                                    />
                                }
                            },
                            CrudView::Edit(id) => {
                                html! {
                                    <CrudEditView<T>
                                        api_base_url={ctx.props().base_url.clone()}
                                        children={ctx.props().children.clone()}
                                        config={self.config.clone()}
                                        id={id}
                                        list_view_available={true}
                                        on_list={ctx.link().callback(|_| Msg::List)}
                                        on_create={ctx.link().callback(|_| Msg::Create)}
                                        on_delete={ctx.link().callback(Msg::Delete)}
                                    />
                                }
                            },
                        }
                    }

                    {
                        match &self.entity_to_delete {
                            Some(entity) => html! {
                                <CrudModal>
                                    <CrudDeleteModal<T>
                                        entity={entity.clone()}
                                        on_cancel={ctx.link().callback(|_| Msg::DeleteCanceled)}
                                        on_delete={ctx.link().callback(|_| Msg::DeleteApproved)}
                                    />
                                </CrudModal>
                            },
                            None => html! {}
                        }
                    }
                </div>
            </div>
        }
    }
}

impl<T: 'static + CrudDataTrait> Component for CrudInstance<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        //let actions: Vec<Rc<Box<dyn CrudActionTrait>>> = vec![
        //    Rc::new(Box::new(ShowReadViewAction::default())),
        //    Rc::new(Box::new(ShowEditViewAction::default())),
        //    Rc::new(Box::new(DeleteAction::default())),
        //];
        Self {
            instance_dispatch: Dispatch::subscribe(
                ctx.link().callback(Msg::InstanceConfigStoreUpdated),
            ),
            instance_store: Default::default(),
            instance_views_dispatch: Dispatch::subscribe(
                ctx.link().callback(Msg::InstanceViewsStoreUpdated),
            ),
            instance_views_store: Default::default(),

            config: ctx.props().config.clone(),
            entity_to_delete: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::InstanceConfigStoreUpdated(store) => {
                self.instance_store = store;
                match self.instance_store.get(&ctx.props().name) {
                    Some(config) => {
                        self.config = config;
                        true
                    }
                    None => false,
                }
            }
            Msg::InstanceViewsStoreUpdated(store) => {
                self.instance_views_store = store;
                // TODO: Check if we really need to always rerender. Only CreateView needs this...
                true
            }
            Msg::Action((action, _entity)) => {
                log::warn!(
                    "Received action {:?} but no handler was specified for it!",
                    action
                );
                false
            }
            Msg::List => {
                self.set_view(CrudView::List);
                self.store_config(ctx);
                true
            }
            Msg::Create => {
                self.set_view(CrudView::Create);
                self.store_config(ctx);
                true
            }
            Msg::EntityCreated((entity, suggested_view)) => {
                if let Some(suggested_view) = suggested_view {
                    self.set_view(suggested_view);
                } else {
                    self.set_view(CrudView::Edit(entity.get_id()));
                }
                self.store_config(ctx);
                true
            }
            Msg::Read(entity) => {
                self.set_view(CrudView::Read(entity.get_id()));
                self.store_config(ctx);
                true
            }
            Msg::Edit(entity) => {
                self.set_view(CrudView::Edit(entity.get_id()));
                self.store_config(ctx);
                true
            }
            Msg::Delete(entity) => {
                self.entity_to_delete = Some(entity);
                true
            }
            Msg::DeleteCanceled => {
                self.entity_to_delete = None;
                true
            }
            Msg::DeleteApproved => {
                match &self.entity_to_delete {
                    Some(entity) => {
                        let base_url = ctx.props().base_url.clone();
                        let id = entity.get_id();
                        ctx.link().send_future(async move {
                            Msg::Deleted(delete_by_id::<T>(&base_url, DeleteById { id }).await)
                        });
                    },
                    None => log::warn!("Delete was approved, but instance already lost track of the 'entity_to_delete'!"),
                }
                false
            }
            Msg::Deleted(result) => {
                match result {
                    Ok(entity) => match entity {
                        Some(_amount) => {
                            match &self.entity_to_delete {
                                Some(entity) => match self.config.view {
                                    CrudView::Read(id) | CrudView::Edit(id) => {
                                        if id == entity.get_id() {
                                            self.set_view(CrudView::List);
                                            self.store_config(ctx);
                                        }
                                    }
                                    _ => {}
                                },
                                None => {}
                            }
                            self.entity_to_delete = None;
                            true
                        }
                        None => {
                            log::warn!("Server did not respond with an error but also did not send the deleted entity back. Something seems wrong..");
                            false
                        }
                    },
                    Err(err) => {
                        // TODO: Make this error visible in the (still opened) modal window.
                        // Let the user decide what to do.
                        // The user can always click cancel to leave the modal without potential for errors.
                        log::warn!(
                            "Server was unable to delete entity {:?}. Reason: {}",
                            self.entity_to_delete,
                            err
                        );
                        false
                    }
                }
            }
            Msg::OrderBy((field, options)) => {
                self.config.update_order_by(field, options);
                self.store_config(ctx);
                false
            }
            Msg::PageSelected(page) => {
                self.config.page = page;
                self.store_config(ctx);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &ctx.props().portal_target {
            Some(target) => {
                if let Some(portal) = gloo::utils::document().get_element_by_id(target) {
                    create_portal(self.render(ctx), portal.into())
                } else {
                    html! {}
                }
            }
            None => self.render(ctx),
        }
    }
}
