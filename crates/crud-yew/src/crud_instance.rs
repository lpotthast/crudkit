use chrono_utc_date_time::UtcDateTime;
use crud_shared_types::{
    Condition, ConditionClause, ConditionClauseValue, ConditionElement, DeleteResult, Order, Saved,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use uuid::Uuid;
use yew::{
    html::{ChildrenRenderer, Scope},
    prelude::*,
    virtual_dom::VChild,
};
use yewdux::prelude::*;

use crate::{
    services::crud_rest_data_provider::{CrudRestDataProvider, DeleteById},
    DateTimeDisplay,
};

use super::{prelude::*, stores, types::RequestError};

pub enum Msg<T: 'static + CrudMainTrait> {
    InstanceConfigStoreUpdated(Rc<stores::instance::InstanceStore<T>>),
    InstanceViewsStoreUpdated(Rc<stores::instance_views::InstanceViewsStore>),
    ViewLinked(Option<ViewLink<T>>),
    List,
    Create,
    EntityCreated((Saved<T::UpdateModel>, Option<CrudView>)),
    EntityNotCreatedDueToCriticalErrors,
    EntityCreationFailed(RequestError),
    EntityUpdated(Saved<T::UpdateModel>),
    EntityNotUpdatedDueToCriticalErrors,
    EntityUpdateFailed(RequestError),
    Read(T::UpdateModel),
    Edit(T::UpdateModel),
    Delete(T::UpdateModel),
    DeleteCanceled,
    DeleteApproved,
    Deleted(Result<DeleteResult, RequestError>),
    OrderBy((<T::ReadModel as CrudDataTrait>::Field, OrderByUpdateOptions)),
    PageSelected(u64),
    EntityAction((Rc<Box<dyn CrudActionTrait>>, T::ReadModel)),
    GlobalAction(CrudActionAftermath),
    SaveInput((CreateOrUpdateField<T>, Value)),
    GetInput((CreateOrUpdateField<T>, Box<dyn FnOnce(Value)>)),
    Reload,
}

// TODO: Location in source code?
#[derive(Debug, Clone, PartialEq)]
pub enum CreateOrUpdateField<T: CrudMainTrait> {
    CreateField(<T::CreateModel as CrudDataTrait>::Field),
    UpdateField(<T::UpdateModel as CrudDataTrait>::Field),
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
pub struct CrudInstanceConfig<T: CrudMainTrait> {
    pub api_base_url: String,
    pub view: CrudView,
    pub headers: Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions)>,
    // serde bound used as described in: https://github.com/serde-rs/serde/issues/1296
    #[serde(bound = "")]
    pub create_elements: CreateElements<T>,
    // serde bound used as described in: https://github.com/serde-rs/serde/issues/1296
    #[serde(bound = "")]
    pub elements: Vec<Elem<T::UpdateModel>>,
    pub order_by: IndexMap<<T::ReadModel as CrudDataTrait>::Field, Order>,
    pub items_per_page: u64,
    pub page: u64,
    pub nested: Option<NestedConfig>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct CrudStaticInstanceConfig {
    pub actions: Vec<CrudAction>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CreateElements<T: CrudMainTrait> {
    Default,
    Custom(Vec<Elem<T::CreateModel>>),
}

impl<T: CrudMainTrait> Default for CrudInstanceConfig<T> {
    fn default() -> Self {
        let mut order_by = IndexMap::default();
        order_by.insert(T::ReadModel::get_id_field(), Order::Asc);
        Self {
            api_base_url: "".to_owned(),
            view: CrudView::default(),
            headers: vec![(
                T::ReadModel::get_id_field(),
                HeaderOptions {
                    display_name: "ID".to_owned(),
                    min_width: true,
                    ordering_allowed: true,
                    date_time_display: DateTimeDisplay::LocalizedLocal,
                },
            )],
            create_elements: CreateElements::Default,
            elements: vec![],
            order_by,
            items_per_page: 10,
            page: 1,
            nested: None,
        }
    }
}

impl<T: CrudMainTrait> CrudInstanceConfig<T> {
    fn update_order_by(
        &mut self,
        field: <T::ReadModel as CrudDataTrait>::Field,
        options: OrderByUpdateOptions,
    ) {
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
    Relation(VChild<CrudRelation>),
    Select(VChild<CrudResetField>),
}

// Now, we implement `Into<Html>` so that yew knows how to render `Item`.
#[allow(clippy::from_over_into)]
impl Into<Html> for Item {
    fn into(self) -> Html {
        match self {
            Item::NestedInstance(child) => child.into(),
            Item::Relation(child) => child.into(),
            Item::Select(child) => child.into(),
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct Props<T: CrudMainTrait> {
    // TODO: Analyze children once on creation and on prop changes. Pass generated data-structure to children!
    // TODO: Only allow easy-to-parse structure:
    /*
       tbd...
       ListDetails {

       }
       FieldDetails {

       }
    */
    #[prop_or_default]
    pub children: ChildrenRenderer<Item>,
    pub name: String,
    pub config: CrudInstanceConfig<T>,
    pub static_config: CrudStaticInstanceConfig,
    pub portal_target: Option<String>,
}

pub enum ViewLink<T: CrudMainTrait + 'static> {
    List(Scope<CrudListView<T>>),
    Create(Scope<CrudCreateView<T>>),
    Edit(Scope<CrudEditView<T>>),
    Read(Scope<CrudReadView<T>>),
}

pub struct CrudInstance<T: 'static + CrudMainTrait> {
    instance_store: Rc<stores::instance::InstanceStore<T>>,
    instance_dispatch: Dispatch<stores::instance::InstanceStore<T>>,
    instance_views_store: Rc<stores::instance_views::InstanceViewsStore>,
    instance_views_dispatch: Dispatch<stores::instance_views::InstanceViewsStore>,
    instance_links_dispatch: Dispatch<stores::instance_links::InstanceLinksStore<T>>,
    toasts_dispatch: Dispatch<stores::toasts::Toasts>,

    /// Initially `None`, when no view was yet created, otherwise present for 99% of this instances lifetime.
    view_link: Option<ViewLink<T>>,

    config: CrudInstanceConfig<T>,
    static_config: CrudStaticInstanceConfig,
    data_provider: CrudRestDataProvider<T>,
    entity_to_delete: Option<T::UpdateModel>,
    parent_id: Option<u32>,
}

impl<T: 'static + CrudMainTrait> CrudInstance<T> {
    fn store_config(&self, ctx: &Context<CrudInstance<T>>) {
        let name = ctx.props().name.clone();
        let config = self.config.clone();
        self.instance_dispatch
            .reduce_mut(|state| state.save(name, config));

        let name = ctx.props().name.clone();
        let view = self.config.view.clone();
        self.instance_views_dispatch
            .reduce_mut(|state| state.save(name, view));
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
                                        data_provider={self.data_provider.clone()}
                                        children={ctx.props().children.clone()}
                                        config={self.config.clone()}
                                        static_config={self.static_config.clone()}
                                        on_create={ctx.link().callback(|_| Msg::Create)}
                                        on_read={ctx.link().callback(Msg::Read)}
                                        on_edit={ctx.link().callback(Msg::Edit)}
                                        on_delete={ctx.link().callback(Msg::Delete)}
                                        on_order_by={ctx.link().callback(Msg::OrderBy)}
                                        on_page_selected={ctx.link().callback(Msg::PageSelected)}
                                        on_entity_action={ctx.link().callback(Msg::EntityAction)}
                                        on_global_action={ctx.link().callback(Msg::GlobalAction)}
                                        on_link={ctx.link().callback(|link: Option<Scope<CrudListView<T>>>|
                                            Msg::ViewLinked(link.map(|link| ViewLink::List(link))))}
                                    />
                                }
                            },
                            CrudView::Create => {
                                html! {
                                    <CrudCreateView<T>
                                        data_provider={self.data_provider.clone()}
                                        parent_id={self.parent_id.clone()}
                                        children={ctx.props().children.clone()}
                                        config={self.config.clone()}
                                        list_view_available={true}
                                        on_list_view={ctx.link().callback(|_| Msg::List)}
                                        on_entity_created={ctx.link().callback(Msg::EntityCreated)}
                                        on_entity_not_created_critical_errors={ctx.link().callback(|_| Msg::EntityNotCreatedDueToCriticalErrors)}
                                        on_entity_creation_failed={ctx.link().callback(Msg::EntityCreationFailed)}
                                        on_link={ctx.link().callback(|link: Option<Scope<CrudCreateView<T>>>|
                                            Msg::ViewLinked(link.map(|link| ViewLink::Create(link))))}
                                    />
                                }
                            },
                            CrudView::Read(id) => {
                                html! {
                                    <CrudReadView<T>
                                        data_provider={self.data_provider.clone()}
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
                                        data_provider={self.data_provider.clone()}
                                        children={ctx.props().children.clone()}
                                        config={self.config.clone()}
                                        id={id}
                                        list_view_available={true}
                                        on_entity_updated={ctx.link().callback(Msg::EntityUpdated)}
                                        on_entity_not_updated_critical_errors={ctx.link().callback(|_| Msg::EntityNotUpdatedDueToCriticalErrors)}
                                        on_entity_update_failed={ctx.link().callback(Msg::EntityUpdateFailed)}
                                        on_list={ctx.link().callback(|_| Msg::List)}
                                        on_create={ctx.link().callback(|_| Msg::Create)}
                                        on_delete={ctx.link().callback(Msg::Delete)}
                                        on_link={ctx.link().callback(|link: Option<Scope<CrudEditView<T>>>|
                                            Msg::ViewLinked(link.map(|link| ViewLink::Edit(link))))}
                                    />
                                }
                            },
                        }
                    }

                    {
                        match &self.entity_to_delete {
                            Some(entity) => html! {
                                <CrudModal>
                                    <CrudDeleteModal<T::UpdateModel>
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

impl<T: 'static + CrudMainTrait> Component for CrudInstance<T> {
    type Message = Msg<T>;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        if ctx.props().config.api_base_url.is_empty() {
            panic!(
                "api_base_url was not configured for crud instance '{}'",
                ctx.props().name
            );
        }
        //let actions: Vec<Rc<Box<dyn CrudActionTrait>>> = vec![
        //    Rc::new(Box::new(ShowReadViewAction::default())),
        //    Rc::new(Box::new(ShowEditViewAction::default())),
        //    Rc::new(Box::new(DeleteAction::default())),
        //];

        let instance_links_dispatch: Dispatch<stores::instance_links::InstanceLinksStore<T>> =
            Dispatch::new();

        let name = ctx.props().name.clone();
        let link = ctx.link().clone();
        instance_links_dispatch.reduce_mut(|state| state.save(name, Some(link)));

        Self {
            instance_dispatch: Dispatch::subscribe(
                ctx.link().callback(Msg::InstanceConfigStoreUpdated),
            ),
            instance_store: Default::default(),
            instance_views_dispatch: Dispatch::subscribe(
                ctx.link().callback(Msg::InstanceViewsStoreUpdated),
            ),
            instance_views_store: Default::default(),
            instance_links_dispatch,
            toasts_dispatch: Dispatch::new(),

            view_link: None,

            config: ctx.props().config.clone(),
            static_config: ctx.props().static_config.clone(),
            data_provider: CrudRestDataProvider::new(ctx.props().config.api_base_url.clone()),
            entity_to_delete: None,
            parent_id: None,
        }
    }

    fn destroy(&mut self, ctx: &Context<Self>) {
        let name = ctx.props().name.clone();
        self.instance_links_dispatch
            .reduce_mut(|state| state.save(name, None));
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
                // TODO: Do we really need to store this?
                self.instance_views_store = store;

                if let Some(nested) = &ctx.props().config.nested {
                    match self
                        .instance_views_store
                        .get(nested.parent_instance.as_str())
                    {
                        Some(parent_view) => {
                            let parent_id = match parent_view {
                                CrudView::List => None,
                                CrudView::Create => None,
                                CrudView::Read(id) => Some(id),
                                CrudView::Edit(id) => Some(id),
                            };
                            if let Some(id) = parent_id {
                                self.data_provider
                                    .set_base_condition(Some(Condition::All(vec![
                                        ConditionElement::Clause(ConditionClause {
                                            column_name: nested.reference_field.clone(),
                                            operator: crud_shared_types::Operator::Equal,
                                            value: ConditionClauseValue::U32(id),
                                        }),
                                    ])));
                            } else {
                                self.data_provider.set_base_condition(None);
                            }
                            self.parent_id = parent_id;
                            true
                        }
                        None => {
                            // log::info!("no parent config");
                            false
                        }
                    }
                } else {
                    // log::info!("not nested");
                    false
                }
            }
            Msg::ViewLinked(view_link) => {
                self.view_link = view_link;
                false
            }
            Msg::EntityAction((action, _entity)) => {
                log::warn!(
                    "Received action {:?} but no handler was specified for it!",
                    action
                );
                false
            }
            Msg::GlobalAction(action) => {
                if let Some(toast) = action.show_toast {
                    self.toasts_dispatch
                        .reduce_mut(|state| state.push_toast(toast));
                }
                if action.reload_data {
                    ctx.link().send_message(Msg::Reload)
                }
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
            Msg::EntityCreated((save_result, suggested_view)) => {
                if let Some(suggested_view) = suggested_view {
                    self.set_view(suggested_view);
                } else {
                    self.set_view(CrudView::Edit(save_result.entity.get_id()));
                }
                self.store_config(ctx);
                self.toasts_dispatch.reduce_mut(|state| {
                    state.push_toast(Toast {
                        id: Uuid::new_v4(),
                        created_at: UtcDateTime::now(),
                        variant: match save_result.with_validation_errors {
                            true => ToastVariant::Warn,
                            false => ToastVariant::Success,
                        },
                        heading: "Erstellt".to_owned(),
                        message: match save_result.with_validation_errors {
                            true => {
                                "Der Eintrag wurde mit Validierungsfehlern erstellt.".to_owned()
                            }
                            false => "Der Eintrag wurde erfolgreich erstellt.".to_owned(),
                        },
                        dismissible: false,
                        automatically_closing: ToastAutomaticallyClosing::WithDefaultDelay,
                        close_callback: None,
                    })
                });
                true
            }
            Msg::EntityNotCreatedDueToCriticalErrors => {
                self.toasts_dispatch.reduce_mut(move |state| {
                    state.push_toast(Toast {
                        id: Uuid::new_v4(),
                        created_at: UtcDateTime::now(),
                        variant: ToastVariant::Error,
                        heading: "Nicht erstellt".to_owned(),
                        message: "Kritische Validierungsfehler verhindern das Speichern."
                            .to_owned(),
                        dismissible: false,
                        automatically_closing: ToastAutomaticallyClosing::WithDelay {
                            millis: 3000,
                        },
                        close_callback: None,
                    })
                });
                false
            }
            Msg::EntityCreationFailed(request_error) => {
                self.toasts_dispatch.reduce_mut(move |state| {
                    state.push_toast(Toast {
                        id: Uuid::new_v4(),
                        created_at: UtcDateTime::now(),
                        variant: ToastVariant::Error,
                        heading: "Nicht erstellt".to_owned(),
                        message: format!(
                            "Der Eintrag konnte nicht erstellt werden: {}.",
                            request_error
                        ),
                        dismissible: false,
                        automatically_closing: ToastAutomaticallyClosing::WithDelay {
                            millis: 3000,
                        },
                        close_callback: None,
                    })
                });
                false
            }
            Msg::EntityUpdated(save_result) => {
                self.toasts_dispatch.reduce_mut(|state| {
                    state.push_toast(Toast {
                        id: Uuid::new_v4(),
                        created_at: UtcDateTime::now(),
                        variant: match save_result.with_validation_errors {
                            true => ToastVariant::Warn,
                            false => ToastVariant::Success,
                        },
                        heading: String::from("Gespeichert"),
                        message: match save_result.with_validation_errors {
                            true => {
                                String::from("Eintrag wurde mit Validierungsfehlern gespeichert.")
                            }
                            false => String::from("Eintrag wurde erfolgreich gespeichert."),
                        },
                        dismissible: false,
                        automatically_closing: ToastAutomaticallyClosing::WithDefaultDelay,
                        close_callback: None,
                    })
                });
                true
            }
            Msg::EntityNotUpdatedDueToCriticalErrors => {
                self.toasts_dispatch.reduce_mut(move |state| {
                    state.push_toast(Toast {
                        id: Uuid::new_v4(),
                        created_at: UtcDateTime::now(),
                        variant: ToastVariant::Error,
                        heading: "Nicht aktualisiert".to_owned(),
                        message: "Kritische Validierungsfehler verhindern das Speichern."
                            .to_owned(),
                        dismissible: false,
                        automatically_closing: ToastAutomaticallyClosing::WithDelay {
                            millis: 3000,
                        },
                        close_callback: None,
                    })
                });
                false
            }
            Msg::EntityUpdateFailed(request_error) => {
                self.toasts_dispatch.reduce_mut(move |state| {
                    state.push_toast(Toast {
                        id: Uuid::new_v4(),
                        created_at: UtcDateTime::now(),
                        variant: ToastVariant::Error,
                        heading: "Nicht aktualisiert".to_owned(),
                        message: format!(
                            "Der Eintrag konnte nicht aktualisiert werden: {}.",
                            request_error
                        ),
                        dismissible: false,
                        automatically_closing: ToastAutomaticallyClosing::WithDelay {
                            millis: 3000,
                        },
                        close_callback: None,
                    })
                });
                false
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
                        let id = entity.get_id();
                        let data_provider = self.data_provider.clone();
                        ctx.link().send_future(async move {
                            Msg::Deleted(data_provider.delete_by_id(DeleteById { id }).await)
                        });
                    },
                    None => log::warn!("Delete was approved, but instance already lost track of the 'entity_to_delete'!"),
                }
                false
            }
            Msg::Deleted(result) => {
                match result {
                    Ok(delete_result) => match delete_result {
                        DeleteResult::Deleted(_amount) => {
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
                        DeleteResult::CriticalValidationErrors => {
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
            Msg::SaveInput((field, value)) => {
                // log::info!(
                //     "CrudInstance saving value '{:?}' for field '{:?}'",
                //     value,
                //     field
                // );
                if let Some(view_link) = &self.view_link {
                    match view_link {
                        ViewLink::List(_link) => {
                            log::warn!("Ignoring 'SaveInput' message as we are currently in the list view, which is unable to save any user inputs.");
                        }
                        ViewLink::Create(link) => match field {
                            CreateOrUpdateField::CreateField(field) => {
                                link.send_message(
                                        <CrudCreateView<T> as Component>::Message::CreateModelFieldChanged(
                                            (field, value),
                                        ),
                                    );
                            }
                            CreateOrUpdateField::UpdateField(field) => {
                                link.send_message(
                                        <CrudCreateView<T> as Component>::Message::UpdateModelFieldChanged(
                                            (field, value),
                                        ),
                                    );
                            }
                        },
                        ViewLink::Edit(link) => match field {
                            CreateOrUpdateField::CreateField(field) => {
                                log::error!("CrudInstance: Cannot 'SaveInput' from 'CreateModel' field '{field:?}' when being in the edit view. You must declare an 'EditModel' field for this view.")
                            }
                            CreateOrUpdateField::UpdateField(field) => {
                                link.send_message(
                                    <CrudEditView<T> as Component>::Message::ValueChanged((
                                        field, value,
                                    )),
                                );
                            }
                        },
                        ViewLink::Read(_link) => {
                            log::warn!("Ignoring 'SaveInput' message as we are currently in the read view, which is unable to save any user inputs.");
                        }
                    }
                } else {
                    log::warn!("Could not forward SaveInput message, as no view link was registered in instance {:?}.", self.config);
                }
                false
            }
            Msg::GetInput((field, receiver)) => {
                if let Some(view_link) = &self.view_link {
                    match view_link {
                        ViewLink::List(_link) => {
                            log::warn!("Ignoring 'GetInput' message as we are currently in the list view, which is unable to retrieve any user inputs.");
                        }
                        ViewLink::Create(link) => match field {
                            CreateOrUpdateField::CreateField(field) => {
                                link.send_message(
                                        <CrudCreateView<T> as Component>::Message::GetCreateModelFieldValue(
                                            (field.clone(), receiver),
                                        ),
                                    );
                            }
                            CreateOrUpdateField::UpdateField(field) => {
                                link.send_message(
                                        <CrudCreateView<T> as Component>::Message::GetUpdateModelFieldValue(
                                            (field.clone(), receiver),
                                        ),
                                    );
                            }
                        },
                        ViewLink::Edit(link) => match field {
                            CreateOrUpdateField::CreateField(field) => {
                                log::error!("CrudInstance: Cannot 'GetInput' from 'CreateModel' field '{field:?}' when being in the edit view. You must declare an 'EditModel' field for this view.")
                            }
                            CreateOrUpdateField::UpdateField(field) => {
                                link.send_message(
                                    <CrudEditView<T> as Component>::Message::GetInput((
                                        field.clone(),
                                        receiver,
                                    )),
                                );
                            }
                        },
                        ViewLink::Read(_link) => {
                            log::warn!("Ignoring 'GetInput' message as we are currently in the read view, which is unable to retrieve any user inputs.");
                        }
                    }
                } else {
                    log::warn!("Could not forward GetInput message, as neither a create view nor an edit view registered.");
                }
                false
            }
            Msg::Reload => {
                // TODO: Can we also reload in create, edit or read view?
                if let Some(view_link) = &self.view_link {
                    match view_link {
                        ViewLink::List(link) => {
                            link.send_message(<CrudListView<T> as Component>::Message::Reload)
                        }
                        ViewLink::Create(_link) => todo!(),
                        ViewLink::Edit(_link) => todo!(),
                        ViewLink::Read(_link) => todo!(),
                    }
                }
                true
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
