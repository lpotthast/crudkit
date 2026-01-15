use crate::dynamic::crud_action::CrudActionAftermath;
use crate::dynamic::crud_create_view::CrudCreateView;
use crate::dynamic::crud_delete_many_modal::CrudDeleteManyModal;
use crate::dynamic::crud_delete_modal::CrudDeleteModal;
use crate::dynamic::crud_edit_view::CrudEditView;
use crate::dynamic::crud_instance_config::{
    CrudInstanceConfig, CrudMutableInstanceConfig, CrudParentConfig, CrudStaticInstanceConfig,
};
use crate::dynamic::crud_list_view::CrudListView;
use crate::dynamic::crud_read_view::CrudReadView;
use crate::shared::crud_instance_config::{ItemsPerPage, PageNr};
use crate::shared::crud_instance_mgr::{CrudInstanceMgrContext, InstanceState};
use crudkit_condition::{Condition, ConditionElement};
use crudkit_core::{Deleted, DeletedMany, Order};
use crudkit_id::SerializableId;
use crudkit_web::dynamic::prelude::*;
use crudkit_web::dynamic::{AnyReadField, AnyReadModel, AnyReadOrUpdateModel};
use crudkit_web::request_error::CrudOperationError;
use indexmap::IndexMap;
use leptonic::components::prelude::*;
use leptos::prelude::*;
use std::sync::Arc;
use time::OffsetDateTime;
use uuid::Uuid;

/// Runtime data of this instance, provided to child components through provide_context.
///
/// This context struct contains data not really necessary in every view,
/// but as we want to retain all state between view changes, this is a reasonable place to store that state.
/// It allows a user to configure the list view, update an entry, return, and then find the list view unaltered.
///
/// Signal setters should generally not be pub. Define custom functions providing the required functionality.
#[derive(Debug, Clone, Copy)]
pub struct CrudInstanceContext {
    default_config: StoredValue<CrudMutableInstanceConfig>,
    pub(crate) static_config: StoredValue<CrudStaticInstanceConfig>,

    /// The current "view" of this instance. Can be List, Create, Edit, Read, ... Acts like a router...
    pub view: ReadSignal<SerializableCrudView>,
    set_view: WriteSignal<SerializableCrudView>,

    /// The page the user is currently on in the list view.
    pub current_page: ReadSignal<PageNr>,
    set_current_page: WriteSignal<PageNr>,

    /// The amount of items shown per page in the list view.
    pub items_per_page: ReadSignal<ItemsPerPage>,
    set_items_per_page: WriteSignal<ItemsPerPage>,

    /// How data should be ordered when querying data for the ist view.
    pub order_by: ReadSignal<IndexMap<AnyReadField, Order>>,
    set_order_by: WriteSignal<IndexMap<AnyReadField, Order>>,

    /// Configuration of a parent, if present.
    pub parent: StoredValue<Option<CrudParentConfig>>,

    /// If a parent is referenced, this may provide the id the parent is currently using.
    pub parent_id: Signal<Option<SerializableId>>,

    /// If a parent is referenced and that parent currently provides an id,
    /// this hold a condition restraining the current resource to elements referencing the parent id.
    pub parent_id_referencing_condition: Signal<Option<crudkit_condition::Condition>>,

    /// The base condition applicable when fetching data.
    pub base_condition: Signal<Option<crudkit_condition::Condition>>,

    /// Whenever the user requests to delete something, this is the place that information is stored.
    pub deletion_request: ReadSignal<Option<AnyReadOrUpdateModel>>,
    set_deletion_request: WriteSignal<Option<AnyReadOrUpdateModel>>,

    /// Whenever the user requests to delete multiple entities, this stores the entities to delete.
    pub mass_deletion_request: ReadSignal<Option<Arc<Vec<AnyReadModel>>>>,
    set_mass_deletion_request: WriteSignal<Option<Arc<Vec<AnyReadModel>>>>,

    /// Whenever this signal changes, the current view should "refresh" by reloading all server provided data.
    /// It simply provides a new random ID on each invocation.
    pub reload: ReadSignal<Uuid>,
    set_reload: WriteSignal<Uuid>,
}

impl CrudInstanceContext {
    /// Opens the list view.
    pub fn list(&self) {
        self.set_view.set(SerializableCrudView::List);
    }

    /// Opens the create view.
    pub fn create(&self) {
        self.set_view.set(SerializableCrudView::Create);
    }

    /// Opens the read view for the given entity.
    pub fn read(&self, entity_id: SerializableId) {
        self.set_view.set(SerializableCrudView::Read(entity_id));
    }

    /// Opens the edit view for the given entity.
    pub fn edit(&self, entity_id: SerializableId) {
        self.set_view.set(SerializableCrudView::Edit(entity_id));
    }

    pub fn set_page(&self, page_number: PageNr) {
        self.set_current_page.set(page_number);
    }

    pub fn set_items_per_page(&self, items_per_page: ItemsPerPage) {
        self.set_items_per_page.set(items_per_page);
    }

    // TODO: Why is this here and CrudInstanceConfig#update_order_by exists?
    pub fn oder_by(&self, field: AnyReadField, options: OrderByUpdateOptions) {
        self.set_order_by
            .update(|order_by: &mut IndexMap<AnyReadField, Order>| {
                let prev = order_by.get(&field).cloned();
                tracing::debug!(?field, ?options, "order by");
                if !options.append {
                    order_by.clear();
                }
                order_by.insert(
                    field,
                    match prev {
                        Some(order) => match order {
                            Order::Asc => Order::Desc,
                            Order::Desc => Order::Asc,
                        },
                        None => Order::Asc,
                    },
                );
            })
    }

    pub fn tab_selected(&self, tab_id: TabId) {
        tracing::info!(?tab_id, "tab_selected");
    }

    pub fn request_deletion_of(&self, entity: AnyReadOrUpdateModel) {
        // TODO: Use upcasting instead of helper function when Rust 1.86 lands. (see: dyn upcasting coercion")
        self.set_deletion_request.set(Some(entity));
    }

    pub fn request_mass_deletion(&self, entities: Arc<Vec<AnyReadModel>>) {
        if !entities.is_empty() {
            self.set_mass_deletion_request.set(Some(entities));
        }
    }

    pub fn cancel_mass_deletion(&self) {
        self.set_mass_deletion_request.set(None);
    }

    // TODO: Other functions do not take a . Should the instance provide its  to store it in this context? Would allow everyone to have access.
    pub fn handle_action_outcome(&self, outcome: Result<CrudActionAftermath, CrudActionAftermath>) {
        tracing::info!(?outcome, "handling action outcome");

        let CrudActionAftermath {
            show_toast,
            reload_data,
        } = match outcome {
            Ok(outcome) => outcome,
            Err(outcome) => outcome,
        };

        if let Some(toast) = show_toast {
            expect_context::<Toasts>().push(toast);
        }

        if reload_data {
            self.reload();
        }
    }

    pub fn reload(&self) {
        self.set_reload.set(Uuid::new_v4());
    }

    /// Reset this instance to its default configuration.
    /// Every change made by the user is reverted.
    pub fn reset(&self) {
        let default = self.default_config.get_value();
        self.set_deletion_request.set(None);
        self.set_mass_deletion_request.set(None);
        self.set_current_page.set(default.page);
        self.set_items_per_page.set(default.items_per_page);
        self.set_order_by.set(default.order_by.clone());
        // TODO: Should there be functions resetting individual views? This always resets everything and sets the view to be the List view...
        self.set_view.set(default.view);
    }
}

// TODO: Effect::new over all signals in config, bundle, serialize and store...

#[component]
pub fn CrudInstance(
    name: &'static str,
    config: CrudInstanceConfig,
    #[prop(optional)] parent: Option<CrudParentConfig>,
    #[prop(optional)] on_context_created: Option<Callback<CrudInstanceContext>>,
) -> impl IntoView {
    let (config, static_config) = config.split();

    let static_config = StoredValue::new(static_config);

    let (api_base_url, set_api_base_url) = signal(config.api_base_url.clone());
    let (view, set_view) = signal(config.view.clone());
    let serializable_view = Memo::<SerializableCrudView>::new(move |_| view.get().into()); // TODO: remove this. now irrelevant

    let mgr = expect_context::<CrudInstanceMgrContext>();
    mgr.register(
        name,
        InstanceState {
            name,
            view: serializable_view.into(),
        },
    );

    let (headers, _set_headers) = signal(config.headers.clone());
    let (current_page, set_current_page) = signal(config.page.clone());
    let (items_per_page, set_items_per_page) = signal(config.items_per_page.clone());
    let (order_by, set_order_by) = signal(config.order_by.clone());

    let parent = StoredValue::new(parent);
    let parent_id = Signal::derive(move || {
        parent
            .read_value()
            .as_ref()
            .and_then(|parent| get_parent_id(&parent, mgr))
    });
    let parent_id_referencing_condition = Signal::derive(move || {
        parent
            .read_value()
            .as_ref()
            .and_then(|parent| get_parent_id(&parent, mgr).map(|id| (parent, id)))
            .map(|(parent, id)| {
                let (_name, value) =
                    id.0.into_iter()
                        .find(|(id_field_name, _id_field_value)| {
                            id_field_name == &parent.referenced_field
                        })
                        .expect("referenced field to be an ID field.");
                crudkit_condition::Condition::All(vec![
                    crudkit_condition::ConditionElement::Clause(
                        crudkit_condition::ConditionClause {
                            column_name: parent.referencing_field.clone(),
                            operator: crudkit_condition::Operator::Equal,
                            value: value.clone().try_into().unwrap(),
                        },
                    ),
                ])
            })
    });
    let base_condition = config.base_condition.clone();
    let base_condition = Signal::derive(move || {
        crudkit_condition::merge_conditions(
            parent_id_referencing_condition.get(),
            base_condition.clone(),
        )
    });
    let (create_elements, _set_create_elements) = signal(config.create_elements.clone());
    let (update_elements, _set_update_elements) = signal(config.elements.clone());
    let (deletion_request, set_deletion_request) = signal(None);
    let (mass_deletion_request, set_mass_deletion_request) = signal(None::<Arc<Vec<AnyReadModel>>>);
    let (reload, set_reload) = signal(Uuid::new_v4());

    let default_config = StoredValue::new(config);

    let data_provider = Signal::derive(move || {
        CrudRestDataProvider::new(
            api_base_url.get(),
            static_config.read_value().reqwest_executor.clone(),
            static_config.read_value().resource_name.clone(),
        )
    });

    // ctx is copy. But is it efficient? Do we want to put this into a stored value instead?
    let ctx = CrudInstanceContext {
        default_config,
        static_config,
        view,
        set_view,
        current_page,
        set_current_page,
        items_per_page,
        set_items_per_page,
        order_by,
        set_order_by,
        parent,
        parent_id,
        parent_id_referencing_condition,
        base_condition,
        deletion_request,
        set_deletion_request,
        mass_deletion_request,
        set_mass_deletion_request,
        reload,
        set_reload,
    };
    provide_context(ctx);
    if let Some(on_context_created) = on_context_created {
        on_context_created.run(ctx)
    }

    let read_field_renderer_registry =
        Signal::derive(move || static_config.read_value().read_field_renderer.clone());
    let create_field_renderer_registry =
        Signal::derive(move || static_config.read_value().create_field_renderer.clone());
    let update_field_renderer_registry =
        Signal::derive(move || static_config.read_value().update_field_renderer.clone());

    let actions = Signal::derive(move || static_config.read_value().actions.clone());
    let entity_actions = Signal::derive(move || static_config.read_value().entity_actions.clone());

    let on_cancel_delete = Callback::new(move |()| {
        tracing::info!("Removing delete request");
        set_deletion_request.set(None);
    });

    let delete_action = Action::new_local(move |entity_id: &SerializableId| {
        let data_provider = data_provider.get();
        let id = entity_id.clone();
        async move {
            let result = data_provider.delete_by_id(DeleteById { id }).await;

            // The delete operation was performed and must therefore no longer be requested.
            set_deletion_request.set(None);

            // No matter where the user deleted an entity, the list view should be shown afterwards.
            ctx.list();

            // The user must be notified how the delete operation went.
            handle_delete_result(result);

            // We have to reload the list-view!
            ctx.reload();
        }
    });

    let on_accept_delete = Callback::new(move |entity: AnyReadOrUpdateModel| {
        let id = match entity {
            AnyReadOrUpdateModel::Read(model) => model.get_id(),
            AnyReadOrUpdateModel::Update(model) => model.get_id(),
        };
        delete_action.dispatch(id);
    });

    let on_cancel_delete_many = Callback::new(move |()| {
        tracing::info!("Removing mass delete request");
        set_mass_deletion_request.set(None);
    });

    let delete_many_action = Action::new_local(move |entities: &Arc<Vec<AnyReadModel>>| {
        let data_provider = data_provider.get();
        let entities = entities.clone();
        async move {
            // Build a condition from all selected entity IDs.
            // Each entity's ID fields become an AND condition, and all entities are OR'd together.
            let condition = build_condition_from_entities(&entities);

            let result = data_provider
                .delete_many(DeleteMany {
                    condition: Some(condition),
                })
                .await;

            // The delete operation was performed and must therefore no longer be requested.
            set_mass_deletion_request.set(None);

            // The user must be notified how the delete operation went.
            handle_delete_many_result(result);

            // We have to reload the list-view!
            ctx.reload();
        }
    });

    let on_accept_delete_many = Callback::new(move |entities: Arc<Vec<AnyReadModel>>| {
        delete_many_action.dispatch(entities);
    });

    view! {
        <div class="crud-instance">
            <div class="body">
                {move || match view.get() {
                    SerializableCrudView::List => view! {
                        <CrudListView
                            data_provider=data_provider
                            headers=headers
                            order_by=order_by
                            field_renderer_registry=read_field_renderer_registry
                            actions=actions
                        />
                    }.into_any(),
                    SerializableCrudView::Create => view! {
                        <CrudCreateView
                            data_provider=data_provider
                            create_elements=create_elements
                            field_renderer_registry=create_field_renderer_registry
                            on_edit_view=move |id| ctx.edit(id)
                            on_list_view=move || ctx.list()
                            on_create_view=move || ctx.create()
                            on_entity_created=move |_saved| {}
                            on_entity_creation_failed=move |_error: CrudOperationError| {
                                // TODO: Handle the error: Display notification to the user.
                            }
                            on_tab_selected=move |tab_id| {
                                ctx.tab_selected(tab_id)
                            }
                        />
                    }.into_any(),
                    SerializableCrudView::Read(id) => view! {
                        <CrudReadView
                            id=id
                            data_provider=data_provider
                            actions=entity_actions
                            elements=update_elements
                            field_renderer_registry=update_field_renderer_registry
                            on_list_view=move || ctx.list()
                            on_tab_selected=move |tab_id| {
                                ctx.tab_selected(tab_id)
                            }
                        />
                    }.into_any(),
                    SerializableCrudView::Edit(id) => view! {
                        <CrudEditView
                            id=id
                            data_provider=data_provider
                            actions=entity_actions
                            elements=update_elements
                            field_renderer_registry=update_field_renderer_registry
                            on_list_view=move || ctx.list()
                            on_create_view=move || ctx.create()
                            on_entity_updated=move |_saved| {}
                            // TODO: Do we even need this callback? Deletion is handled inside this (CrudInstance) component using/inside of `delete_action`. We dont have an on_entity_delete_failed here. This seems somewhat inconsistent.
                            on_entity_update_failed=move |_error: CrudOperationError| {
                                // TODO: Handle the error: Display notification to the user.
                            }
                            on_tab_selected=move |tab_id| {
                                ctx.tab_selected(tab_id)
                            }
                        />
                    }.into_any(),
                }}
                <CrudDeleteModal
                    entity=deletion_request
                    on_cancel=on_cancel_delete.clone()
                    on_accept=on_accept_delete.clone()
                />
                <CrudDeleteManyModal
                    entities=mass_deletion_request
                    on_cancel=on_cancel_delete_many.clone()
                    on_accept=on_accept_delete_many.clone()
                />
            </div>
        </div>
    }
}

fn get_parent_id(parent: &CrudParentConfig, mgr: CrudInstanceMgrContext) -> Option<SerializableId> {
    let parent_state = mgr
        .instances
        // Must be an untracked access!
        // Otherwise, at instance nesting depth 3, rendering the instance and registering it would
        // case instance at depth 2 to register this change here and force a field-rerender.
        .read_untracked()
        .get_by_name(parent.name)
        .expect("parent to be managed");
    match parent_state.view.get_untracked() {
        SerializableCrudView::List => None,
        SerializableCrudView::Create => None,
        SerializableCrudView::Read(id) => Some(id),
        SerializableCrudView::Edit(id) => Some(id),
    }
}

fn handle_delete_result(result: Result<Deleted, RequestError>) {
    match result {
        Ok(deleted) => {
            let num = deleted.entities_affected;
            expect_context::<Toasts>().push(Toast {
                id: Uuid::new_v4(),
                created_at: OffsetDateTime::now_utc(),
                variant: ToastVariant::Success,
                header: ViewFn::from(|| "Löschen"),
                body: ViewFn::from(move || {
                    format!(
                        "{num} {} erfolgreich gelöscht.",
                        match num {
                            1 => "Eintrag",
                            _ => "Einträge",
                        }
                    )
                }),
                timeout: ToastTimeout::DefaultDelay,
            })
        }
        Err(err) => {
            let error = CrudOperationError::from(err);
            match &error {
                CrudOperationError::Forbidden { reason } => {
                    let reason = reason.clone();
                    expect_context::<Toasts>().push(Toast {
                        id: Uuid::new_v4(),
                        created_at: OffsetDateTime::now_utc(),
                        variant: ToastVariant::Warn,
                        header: ViewFn::from(|| "Löschen"),
                        body: ViewFn::from(move || {
                            format!("Löschvorgang abgebrochen. Grund: {reason}")
                        }),
                        timeout: ToastTimeout::DefaultDelay,
                    });
                }
                CrudOperationError::UnprocessableEntity { reason } => {
                    let reason = reason.clone();
                    expect_context::<Toasts>().push(Toast {
                        id: Uuid::new_v4(),
                        created_at: OffsetDateTime::now_utc(),
                        variant: ToastVariant::Warn,
                        header: ViewFn::from(|| "Löschen"),
                        body: ViewFn::from(move || {
                            format!("Löschvorgang nicht möglich. Grund: {reason}")
                        }),
                        timeout: ToastTimeout::DefaultDelay,
                    });
                }
                _ => {
                    expect_context::<Toasts>().push(Toast {
                        id: Uuid::new_v4(),
                        created_at: OffsetDateTime::now_utc(),
                        variant: ToastVariant::Error,
                        header: ViewFn::from(|| "Löschen"),
                        body: ViewFn::from(move || {
                            format!("Konnte Eintrag nicht Löschen: {error}")
                        }),
                        timeout: ToastTimeout::DefaultDelay,
                    });
                }
            }
        }
    }
}

// TODO: move below function somewhere more appropriate!

/// Build a condition from a list of entities.
/// Each entity's ID fields are combined with AND, and all entities are combined with OR.
fn build_condition_from_entities(entities: &[AnyReadModel]) -> Condition {
    use crudkit_condition::TryIntoAllEqualCondition;

    if entities.is_empty() {
        return Condition::none();
    }

    let entity_conditions: Vec<Condition> = entities
        .iter()
        .filter_map(|entity| {
            let id = entity.get_id();
            match id.0.into_iter().try_into_all_equal_condition() {
                Ok(condition) => Some(condition),
                Err(err) => {
                    tracing::warn!(?err, "Could not convert entity ID to condition");
                    None
                }
            }
        })
        .collect();

    if entity_conditions.is_empty() {
        return Condition::none();
    }

    // All entities are OR'd together.
    if entity_conditions.len() == 1 {
        entity_conditions.into_iter().next().expect("checked above")
    } else {
        Condition::Any(
            entity_conditions
                .into_iter()
                .map(|c| ConditionElement::Condition(Box::new(c)))
                .collect(),
        )
    }
}

fn handle_delete_many_result(result: Result<DeletedMany, RequestError>) {
    match result {
        Ok(delete_result) => {
            let deleted = delete_result.deleted_count;
            let aborted = delete_result.aborted.len();
            let validation_failed = delete_result.validation_failed.len();
            let errors = delete_result.errors.len();

            if deleted > 0 && aborted == 0 && validation_failed == 0 && errors == 0 {
                // Complete success
                expect_context::<Toasts>().push(Toast {
                    id: Uuid::new_v4(),
                    created_at: OffsetDateTime::now_utc(),
                    variant: ToastVariant::Success,
                    header: ViewFn::from(|| "Löschen"),
                    body: ViewFn::from(move || {
                        format!(
                            "{deleted} {} erfolgreich gelöscht.",
                            match deleted {
                                1 => "Eintrag",
                                _ => "Einträge",
                            }
                        )
                    }),
                    timeout: ToastTimeout::DefaultDelay,
                });
            } else if deleted > 0 {
                // Partial success
                expect_context::<Toasts>().push(Toast {
                    id: Uuid::new_v4(),
                    created_at: OffsetDateTime::now_utc(),
                    variant: ToastVariant::Warn,
                    header: ViewFn::from(|| "Löschen"),
                    body: ViewFn::from(move || {
                        let mut msg = format!(
                            "{deleted} {} gelöscht.",
                            match deleted {
                                1 => "Eintrag",
                                _ => "Einträge",
                            }
                        );
                        if aborted > 0 {
                            msg.push_str(&format!(" {aborted} abgebrochen."));
                        }
                        if validation_failed > 0 {
                            msg.push_str(&format!(" {validation_failed} Validierungsfehler."));
                        }
                        if errors > 0 {
                            msg.push_str(&format!(" {errors} Fehler."));
                        }
                        msg
                    }),
                    timeout: ToastTimeout::DefaultDelay,
                });
            } else {
                // Complete failure
                expect_context::<Toasts>().push(Toast {
                    id: Uuid::new_v4(),
                    created_at: OffsetDateTime::now_utc(),
                    variant: ToastVariant::Error,
                    header: ViewFn::from(|| "Löschen"),
                    body: ViewFn::from(move || {
                        let mut msg = String::from("Keine Einträge gelöscht.");
                        if aborted > 0 {
                            msg.push_str(&format!(" {aborted} abgebrochen."));
                        }
                        if validation_failed > 0 {
                            msg.push_str(&format!(" {validation_failed} Validierungsfehler."));
                        }
                        if errors > 0 {
                            msg.push_str(&format!(" {errors} Fehler."));
                        }
                        msg
                    }),
                    timeout: ToastTimeout::DefaultDelay,
                });
            }
        }
        Err(err) => {
            expect_context::<Toasts>().push(Toast {
                id: Uuid::new_v4(),
                created_at: OffsetDateTime::now_utc(),
                variant: ToastVariant::Error,
                header: ViewFn::from(|| "Löschen"),
                body: ViewFn::from(move || format!("Konnte Einträge nicht Löschen: {err:#?}")),
                timeout: ToastTimeout::DefaultDelay,
            });
        }
    }
}
