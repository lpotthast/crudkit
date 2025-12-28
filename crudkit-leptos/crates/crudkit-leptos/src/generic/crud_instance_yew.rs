use crudkit_condition::{Condition, ConditionClause, ConditionElement};
use crudkit_id::{Id, SerializableId};
use crudkit_shared::{DeleteResult, Saved};

use std::rc::Rc;
use tracing::{error, info, warn};
use uuid::Uuid;
use yew::{
    html::{ChildrenRenderer, Scope},
    prelude::*,
    virtual_dom::VChild,
};
use yewdux::prelude::*;

use crate::{prelude::*, stores};

pub enum Msg<T: 'static + CrudMainTrait> {
    InstanceConfigStoreUpdated(Rc<stores::instance::InstanceStore<T>>),
    InstanceViewsStoreUpdated(Rc<stores::instance_views::InstanceViewsStore>),
    ViewLinked(Option<ViewLink<T>>),
    List,
    Create,
    EntityCreated(
        (
            Saved<T::UpdateModel>,
            Option<CrudView<T::ReadModelId, T::UpdateModelId>>,
        ),
    ),
    EntityCreationAborted(String),
    EntityNotCreatedDueToCriticalErrors,
    EntityCreationFailed(RequestError),
    EntityUpdated(Saved<T::UpdateModel>),
    EntityUpdateAborted(String),
    EntityNotUpdatedDueToCriticalErrors,
    EntityUpdateFailed(RequestError),
    Read(T::ReadModel),
    Edit(T::UpdateModel),
    Delete(DeletableModel<T::ReadModel, T::UpdateModel>),
    DeleteCanceled,
    DeleteApproved,
    Deleted(Result<DeleteResult, RequestError>),
    OrderBy((<T::ReadModel as CrudDataTrait>::Field, OrderByUpdateOptions)),
    PageSelected(u64),
    ItemCountSelected(u64),
    TabSelected(Label),
    EntityAction((Rc<Box<dyn CrudActionTrait>>, T::ReadModel)),
    CustomEntityAction(CrudActionAftermath),
    GlobalAction(CrudActionAftermath),
    /// Save input for a field or set this field into its error state.
    SaveInput((CreateOrUpdateField<T>, Result<Value, String>)),
    GetInput((CreateOrUpdateField<T>, Box<dyn FnOnce(Value)>)),
    Reset,
    Reload,
}

// TODO: Location in source code?
#[derive(Debug, Clone, PartialEq)]
pub enum CreateOrUpdateField<T: CrudMainTrait> {
    CreateField(<T::CreateModel as CrudDataTrait>::Field),
    UpdateField(<T::UpdateModel as CrudDataTrait>::Field),
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
    pub static_config: CrudStaticInstanceConfig<T>,
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
    static_config: CrudStaticInstanceConfig<T>,
    data_provider: CrudRestDataProvider<T>,
    entity_to_delete: Option<DeletableModel<T::ReadModel, T::UpdateModel>>,
    parent_id: Option<SerializableId>,
}

impl<T: 'static + CrudMainTrait> CrudInstance<T> {
    fn store_config(&self, ctx: &Context<CrudInstance<T>>) {
        let name = ctx.props().name.clone();
        let config = self.config.clone();
        self.instance_dispatch
            .reduce_mut(|state| state.save(name, config));

        let name = ctx.props().name.clone();
        let view = self.config.view.clone();

        let serializable_view: SerializableCrudView = view.into();

        self.instance_views_dispatch
            .reduce_mut(|state| state.save(name, serializable_view));
    }

    fn set_view(&mut self, view: CrudView<T::ReadModelId, T::UpdateModelId>) {
        self.config.view = view;
    }

    fn render(&self, ctx: &Context<CrudInstance<T>>) -> Html {
        html! {
            <div class={"crud-instance"}>
                <div class={"body"}>
                    {
                        match &self.config.view {
                            CrudView::List => {
                                html! {
                                    <CrudListView<T>
                                        data_provider={self.data_provider.clone()}
                                        children={ctx.props().children.clone()}
                                        custom_fields={self.static_config.custom_read_fields.clone()}
                                        config={self.config.clone()}
                                        static_config={self.static_config.clone()}
                                        on_reset={ctx.link().callback(|_| Msg::Reset)}
                                        on_create={ctx.link().callback(|_| Msg::Create)}
                                        on_read={ctx.link().callback(Msg::Read)}
                                        on_edit={ctx.link().callback(|read_model: T::ReadModel| Msg::Edit(read_model.into()))}
                                        on_delete={ctx.link().callback(|entity| Msg::Delete(DeletableModel::Read(entity)))}
                                        on_order_by={ctx.link().callback(Msg::OrderBy)}
                                        on_page_selected={ctx.link().callback(Msg::PageSelected)}
                                        on_item_count_selected={ctx.link().callback(Msg::ItemCountSelected)}
                                        on_entity_action={ctx.link().callback(Msg::EntityAction)}
                                        on_global_action={ctx.link().callback(Msg::GlobalAction)}
                                        on_link={ctx.link().callback(|link: Option<<CrudListView<T>>>|
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
                                        custom_create_fields={self.static_config.custom_create_fields.clone()}
                                        custom_update_fields={self.static_config.custom_update_fields.clone()}
                                        config={self.config.clone()}
                                        list_view_available={true}
                                        on_list_view={ctx.link().callback(|_| Msg::List)}
                                        on_entity_created={ctx.link().callback(Msg::EntityCreated)}
                                        on_entity_creation_aborted={ctx.link().callback(Msg::EntityCreationAborted)}
                                        on_entity_not_created_critical_errors={ctx.link().callback(|_| Msg::EntityNotCreatedDueToCriticalErrors)}
                                        on_entity_creation_failed={ctx.link().callback(Msg::EntityCreationFailed)}
                                        on_link={ctx.link().callback(|link: Option<<CrudCreateView<T>>>|
                                            Msg::ViewLinked(link.map(|link| ViewLink::Create(link))))}
                                        on_tab_selected={ctx.link().callback(|label| Msg::TabSelected(label))}
                                    />
                                }
                            },
                            CrudView::Read(id) => {
                                html! {
                                    <CrudReadView<T>
                                        data_provider={self.data_provider.clone()}
                                        children={ctx.props().children.clone()}
                                        custom_fields={self.static_config.custom_update_fields.clone()}
                                        config={self.config.clone()}
                                        id={id.clone()}
                                        list_view_available={true}
                                        on_list_view={ctx.link().callback(|_| Msg::List)}
                                        on_tab_selected={ctx.link().callback(|label| Msg::TabSelected(label))}
                                    />
                                }
                            },
                            CrudView::Edit(id) => {
                                html! {
                                    <CrudEditView<T>
                                        data_provider={self.data_provider.clone()}
                                        children={ctx.props().children.clone()}
                                        custom_fields={self.static_config.custom_update_fields.clone()}
                                        config={self.config.clone()}
                                        static_config={self.static_config.clone()}
                                        id={id.clone()}
                                        list_view_available={true}
                                        on_entity_updated={ctx.link().callback(Msg::EntityUpdated)}
                                        on_entity_update_aborted={ctx.link().callback(Msg::EntityUpdateAborted)}
                                        on_entity_not_updated_critical_errors={ctx.link().callback(|_| Msg::EntityNotUpdatedDueToCriticalErrors)}
                                        on_entity_update_failed={ctx.link().callback(Msg::EntityUpdateFailed)}
                                        on_list={ctx.link().callback(|_| Msg::List)}
                                        on_create={ctx.link().callback(|_| Msg::Create)}
                                        on_delete={ctx.link().callback(|entity| Msg::Delete(DeletableModel::Update(entity)))}
                                        on_link={ctx.link().callback(|link: Option<<CrudEditView<T>>>|
                                            Msg::ViewLinked(link.map(|link| ViewLink::Edit(link))))}
                                        on_tab_selected={ctx.link().callback(|label| Msg::TabSelected(label))}
                                        on_entity_action={ctx.link().callback(Msg::CustomEntityAction)}
                                    />
                                }
                            },
                        }
                    }

                    {
                        match &self.entity_to_delete {
                            Some(deletable_model) => match deletable_model {
                                DeletableModel::Read(read_model) => html! {
                                    <CrudModal>
                                        <CrudDeleteModal<T::ReadModel>
                                            entity={read_model.clone()}
                                            on_cancel={ctx.link().callback(|_| Msg::DeleteCanceled)}
                                            on_delete={ctx.link().callback(|_| Msg::DeleteApproved)}>
                                        </CrudDeleteModal<T::ReadModel>>
                                    </CrudModal>
                                },
                                DeletableModel::Update(update_model) => html! {
                                    <CrudModal>
                                        <CrudDeleteModal<T::UpdateModel>
                                            entity={update_model.clone()}
                                            on_cancel={ctx.link().callback(|_| Msg::DeleteCanceled)}
                                            on_delete={ctx.link().callback(|_| Msg::DeleteApproved)}>
                                        </CrudDeleteModal<T::UpdateModel>>
                                    </CrudModal>
                                }
                            }
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
                    // TODO: How do we get the parents ID type?
                    match self
                        .instance_views_store
                        .get(nested.parent_instance.as_str())
                    {
                        Some(parent_view) => {
                            let parent_id = match parent_view {
                                SerializableCrudView::List => None,
                                SerializableCrudView::Create => None,
                                SerializableCrudView::Read(id) => Some(id),
                                SerializableCrudView::Edit(id) => Some(id),
                            };
                            if let Some(parent_id) = &parent_id {
                                let (_field_name, value) = parent_id
                                    .0
                                    .iter()
                                    .find(|(field_name, _value)| {
                                        field_name == nested.parent_field.as_str()
                                    })
                                    .expect("related parent field must be part of the parents id!");

                                self.data_provider
                                    .set_base_condition(Some(Condition::All(vec![
                                        ConditionElement::Clause(ConditionClause {
                                            column_name: nested.reference_field.clone(),
                                            operator: crudkit_condition::Operator::Equal,
                                            value: value.clone().into(),
                                        }),
                                    ])));
                            } else {
                                self.data_provider.set_base_condition(None);
                            }
                            self.parent_id = parent_id.clone();
                            true
                        }
                        None => {
                            info!("no parent config");
                            warn!(
                                "does .instance_views_store.get(nested.parent_instance.as_str()) work correctly now that we have a generic store?"
                            );
                            false
                        }
                    }
                } else {
                    // info!("not nested");
                    false
                }
            }
            Msg::ViewLinked(view_link) => {
                self.view_link = view_link;
                false
            }
            Msg::EntityAction((action, _entity)) => {
                warn!(
                    "Received action {:?} but no handler was specified for it!",
                    action
                );
                false
            }
            Msg::CustomEntityAction(action) => {
                if let Some(toast) = action.show_toast {
                    self.toasts_dispatch
                        .reduce_mut(|state| state.push_toast(toast));
                }
                if action.reload_data {
                    ctx.link().send_message(Msg::Reload)
                }
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
                        created_at: time::OffsetDateTime::now_utc(),
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
            Msg::EntityCreationAborted(reason) => {
                self.toasts_dispatch.reduce_mut(move |state| {
                    state.push_toast(Toast {
                        id: Uuid::new_v4(),
                        created_at: time::OffsetDateTime::now_utc(),
                        variant: ToastVariant::Error,
                        heading: "Nicht erstellt".to_owned(),
                        message: format!("Speichern abgebrochen: {reason}"),
                        dismissible: false,
                        automatically_closing: ToastAutomaticallyClosing::WithDelay {
                            millis: 4000,
                        },
                        close_callback: None,
                    })
                });
                false
            }
            Msg::EntityNotCreatedDueToCriticalErrors => {
                self.toasts_dispatch.reduce_mut(move |state| {
                    state.push_toast(Toast {
                        id: Uuid::new_v4(),
                        created_at: time::OffsetDateTime::now_utc(),
                        variant: ToastVariant::Error,
                        heading: "Nicht erstellt".to_owned(),
                        message: "Kritische Validierungsfehler verhindern das Speichern."
                            .to_owned(),
                        dismissible: false,
                        automatically_closing: ToastAutomaticallyClosing::WithDelay {
                            millis: 4000,
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
                        created_at: time::OffsetDateTime::now_utc(),
                        variant: ToastVariant::Error,
                        heading: "Nicht erstellt".to_owned(),
                        message: format!(
                            "Der Eintrag konnte nicht erstellt werden: {}.",
                            request_error
                        ),
                        dismissible: false,
                        automatically_closing: ToastAutomaticallyClosing::WithDelay {
                            millis: 4000,
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
                        created_at: time::OffsetDateTime::now_utc(),
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
            Msg::EntityUpdateAborted(reason) => {
                self.toasts_dispatch.reduce_mut(move |state| {
                    state.push_toast(Toast {
                        id: Uuid::new_v4(),
                        created_at: time::OffsetDateTime::now_utc(),
                        variant: ToastVariant::Error,
                        heading: "Nicht aktualisiert".to_owned(),
                        message: format!("Speichern abgebrochen: {reason}"),
                        dismissible: false,
                        automatically_closing: ToastAutomaticallyClosing::WithDelay {
                            millis: 4000,
                        },
                        close_callback: None,
                    })
                });
                false
            }
            Msg::EntityNotUpdatedDueToCriticalErrors => {
                self.toasts_dispatch.reduce_mut(move |state| {
                    state.push_toast(Toast {
                        id: Uuid::new_v4(),
                        created_at: time::OffsetDateTime::now_utc(),
                        variant: ToastVariant::Error,
                        heading: "Nicht aktualisiert".to_owned(),
                        message: "Kritische Validierungsfehler verhindern das Speichern."
                            .to_owned(),
                        dismissible: false,
                        automatically_closing: ToastAutomaticallyClosing::WithDelay {
                            millis: 4000,
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
                        created_at: time::OffsetDateTime::now_utc(),
                        variant: ToastVariant::Error,
                        heading: "Nicht aktualisiert".to_owned(),
                        message: format!(
                            "Der Eintrag konnte nicht aktualisiert werden: {}.",
                            request_error
                        ),
                        dismissible: false,
                        automatically_closing: ToastAutomaticallyClosing::WithDelay {
                            millis: 4000,
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
            Msg::Delete(deletable_model) => {
                self.entity_to_delete = Some(deletable_model);
                true
            }
            Msg::DeleteCanceled => {
                self.entity_to_delete = None;
                true
            }
            Msg::DeleteApproved => {
                match &self.entity_to_delete {
                    Some(deletable_model) => {
                        let data_provider = self.data_provider.clone();
                        let serializable_id = match deletable_model {
                            DeletableModel::Read(read_model) => {
                                read_model.get_id().to_serializable_id()
                            }
                            DeletableModel::Update(update_model) => {
                                update_model.get_id().to_serializable_id()
                            }
                        };
                        ctx.link().send_future(async move {
                            Msg::Deleted(
                                data_provider
                                    .delete_by_id(DeleteById {
                                        id: serializable_id,
                                    })
                                    .await,
                            )
                        });
                    }
                    None => warn!(
                        "Delete was approved, but instance already lost track of the 'entity_to_delete'!"
                    ),
                }
                false
            }
            Msg::Deleted(result) => {
                match result {
                    Ok(delete_result) => match delete_result {
                        DeleteResult::Deleted(_amount) => {
                            match &self.entity_to_delete {
                                Some(deletable_model) => match deletable_model {
                                    DeletableModel::Read(read_model) => {
                                        match &self.config.view {
                                            CrudView::Read(id) => {
                                                if id == &read_model.get_id() {
                                                    self.set_view(CrudView::List);
                                                    self.store_config(ctx);
                                                }
                                            }
                                            CrudView::Edit(id) => {
                                                let update_model: T::UpdateModel =
                                                    (*read_model).clone().into();
                                                if id == &update_model.get_id() {
                                                    self.set_view(CrudView::List);
                                                    self.store_config(ctx);
                                                }
                                            }
                                            _ => {}
                                        };
                                    }
                                    DeletableModel::Update(update_model) => {
                                        match &self.config.view {
                                            CrudView::Read(id) => {
                                                // TODO: We cannot do anything here as the UpdateModel cannot be converted into the ReadModel and the ReadModelId cannot be converted into an UpdateModelId...
                                                warn!(
                                                    "possibly needs implementation... crud_instance#Msg::Deleted handler"
                                                );
                                            }
                                            CrudView::Edit(id) => {
                                                if id == &update_model.get_id() {
                                                    self.set_view(CrudView::List);
                                                    self.store_config(ctx);
                                                }
                                            }
                                            _ => {}
                                        };
                                    }
                                },
                                None => {}
                            }
                            self.entity_to_delete = None;
                            true
                        }
                        DeleteResult::Aborted { reason } => {
                            self.toasts_dispatch.reduce_mut(move |state| {
                                state.push_toast(Toast {
                                    id: Uuid::new_v4(),
                                    created_at: time::OffsetDateTime::now_utc(),
                                    variant: ToastVariant::Error,
                                    heading: "Entfernen".to_owned(),
                                    message: format!("Entfernen abgebrochen: {reason}"),
                                    dismissible: false,
                                    automatically_closing: ToastAutomaticallyClosing::WithDelay {
                                        millis: 4000,
                                    },
                                    close_callback: None,
                                })
                            });
                            false
                        }
                        DeleteResult::CriticalValidationErrors => {
                            warn!(
                                "Server did not respond with an error but also did not send the deleted entity back. Something seems wrong.."
                            );
                            false
                        }
                    },
                    Err(err) => {
                        // TODO: Make this error visible in the (still opened) modal window.
                        // Let the user decide what to do.
                        // The user can always click cancel to leave the modal without potential for errors.
                        warn!(
                            "Server was unable to delete entity {:?}. Reason: {}",
                            self.entity_to_delete, err
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
            Msg::ItemCountSelected(items_per_page) => {
                self.config.items_per_page = items_per_page;
                self.store_config(ctx);
                false
            }
            Msg::TabSelected(label) => {
                self.config.active_tab = Some(label);
                self.store_config(ctx);
                false
            }
            Msg::SaveInput((field, value)) => {
                // info!(
                //     "CrudInstance saving value '{:?}' for field '{:?}'",
                //     value,
                //     field
                // );
                if let Some(view_link) = &self.view_link {
                    match view_link {
                        ViewLink::List(_link) => {
                            warn!(
                                "Ignoring 'SaveInput' message as we are currently in the list view, which is unable to save any user inputs."
                            );
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
                                error!(
                                    "CrudInstance: Cannot 'SaveInput' from 'UpdateModel' field '{field:?}' when being in the create view. You must declare an 'CreateModel' field for this view."
                                )
                            }
                        },
                        ViewLink::Edit(link) => match field {
                            CreateOrUpdateField::CreateField(field) => {
                                error!(
                                    "CrudInstance: Cannot 'SaveInput' from 'CreateModel' field '{field:?}' when being in the edit view. You must declare an 'UpdateModel' field for this view."
                                )
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
                            warn!(
                                "Ignoring 'SaveInput' message as we are currently in the read view, which is unable to save any user inputs."
                            );
                        }
                    }
                } else {
                    warn!(
                        "Could not forward SaveInput message, as no view link was registered in instance {:?}.",
                        self.config
                    );
                }
                false
            }
            Msg::GetInput((field, receiver)) => {
                if let Some(view_link) = &self.view_link {
                    match view_link {
                        ViewLink::List(_link) => {
                            warn!(
                                "Ignoring 'GetInput' message as we are currently in the list view, which is unable to retrieve any user inputs."
                            );
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
                                error!(
                                    "CrudInstance: Cannot 'GetInput' from 'UpdateModel' field '{field:?}' when being in the create view. You must declare an 'CreateModel' field for this view."
                                )
                            }
                        },
                        ViewLink::Edit(link) => match field {
                            CreateOrUpdateField::CreateField(field) => {
                                error!(
                                    "CrudInstance: Cannot 'GetInput' from 'CreateModel' field '{field:?}' when being in the edit view. You must declare an 'UpdateModel' field for this view."
                                )
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
                            warn!(
                                "Ignoring 'GetInput' message as we are currently in the read view, which is unable to retrieve any user inputs."
                            );
                        }
                    }
                } else {
                    warn!(
                        "Could not forward GetInput message, as neither a create view nor an edit view registered."
                    );
                }
                false
            }
            Msg::Reset => {
                self.config = ctx.props().config.clone();
                self.static_config = ctx.props().static_config.clone();
                self.store_config(ctx);
                // This will ultimately trigger a rerender, but...
                ctx.link().send_message(Msg::Reload);
                // We have to propagate the new state first, so that the view fetches the correct data (as stated in the default config)!
                true
            }
            Msg::Reload => {
                // TODO: Can we also reload in create, edit or read view?
                if let Some(view_link) = &self.view_link {
                    match view_link {
                        ViewLink::List(link) => {
                            link.send_message(<CrudListView<T> as Component>::Message::Reload)
                        }
                        ViewLink::Create(_link) => todo!(),
                        ViewLink::Edit(link) => {
                            link.send_message(<CrudEditView<T> as Component>::Message::Reload)
                        }
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
