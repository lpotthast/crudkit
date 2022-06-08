use crud_shared_types::Order;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use yew::prelude::*;
use yewdux::prelude::*;

use super::{
    prelude::*,
    services::controller::{delete_by_id, DeleteById},
    stores,
    types::RequestError,
};

pub enum Msg<T: CrudDataTrait> {
    InstanceConfigStoreUpdated(Rc<stores::instance::InstanceStore<T>>),
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
pub struct CrudInstanceConfig<T: CrudDataTrait> {
    pub view: CrudView,
    pub headers: Vec<(T::FieldType, HeaderOptions)>,
    // serde bound used as described in: https://github.com/serde-rs/serde/issues/1296
    #[serde(bound = "")]
    pub elements: Vec<Elem<T>>,
    pub order_by: IndexMap<T::FieldType, Order>,
    pub items_per_page: u64,
    pub page: u64,
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
                },
            )],
            elements: vec![],
            order_by,
            items_per_page: 10,
            page: 1,
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

#[derive(PartialEq, Properties)]
pub struct Props<T: CrudDataTrait> {
    pub name: String,
    pub base_url: String,
    pub config: CrudInstanceConfig<T>,
}

pub struct CrudInstance<T: 'static + CrudDataTrait> {
    config_dispatch: Dispatch<stores::instance::InstanceStore<T>>,
    config: CrudInstanceConfig<T>,
    entity_to_delete: Option<T>,
}

impl<T: 'static + CrudDataTrait> CrudInstance<T> {
    fn store_config(&self, ctx: &Context<CrudInstance<T>>) {
        let name = ctx.props().name.clone();
        let config = self.config.clone();
        self.config_dispatch
            .reduce(|state| state.save(name, config));
    }

    fn set_view(&mut self, view: CrudView) {
        self.config.view = view;
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
        let config_dispatch: Dispatch<stores::instance::InstanceStore<T>> =
            Dispatch::subscribe(ctx.link().callback(Msg::InstanceConfigStoreUpdated));
        Self {
            config_dispatch,
            config: ctx.props().config.clone(),
            entity_to_delete: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::InstanceConfigStoreUpdated(store) => match store.get(&ctx.props().name) {
                Some(config) => {
                    self.config = config;
                    true
                }
                None => false,
            },
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
                                            self.set_view(CrudView::List)
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
        html! {
            <div class={"crud-instance"}>
                <div class={"body"}>
                    {
                        match self.config.view {
                            CrudView::List => {
                                html! {
                                    <CrudListView<T>
                                        base_url={ctx.props().base_url.clone()}
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
                                        base_url={ctx.props().base_url.clone()}
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
                                        base_url={ctx.props().base_url.clone()}
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
                                        base_url={ctx.props().base_url.clone()}
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
