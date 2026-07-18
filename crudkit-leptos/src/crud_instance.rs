//! Mounted CrudKit instance state and rendering.

#![deny(missing_docs)]

use crate::crud_action::CrudActionAftermath;
use crate::crud_create_view::CrudCreateActions;
use crate::crud_delete_many_modal::CrudDeleteManyModal;
use crate::crud_delete_modal::CrudDeleteModal;
use crate::crud_instance_config::{
    CreateElements, CrudInstanceConfig, CrudMutableInstanceConfig, CrudParentConfig,
    CrudStaticInstanceConfig, Header, UpdateElements,
};
use crate::crud_instance_config::{ItemsPerPage, PageNr};
use crate::crud_instance_mgr::{CrudInstanceMgrContext, InstanceState};
use crate::crud_leave_modal::CrudLeaveModal;
use crate::crud_navigation::{CommittedReturnDestination, CrudNavigation};
use crate::crud_view_registry::CrudViewRegistry;
use crudkit_core::condition::{Condition, ConditionClause, ConditionElement};
use crudkit_core::id::{SerializableId, SerializableIdEntry};
use crudkit_core::{Deleted, DeletedMany, Order};
use crudkit_web::prelude::*;
use crudkit_web::request_error::RequestError;
use crudkit_web::view::CrudView;
use crudkit_web::{OrderByUpdateOptions, TabId};
use indexmap::IndexMap;
use leptonic::components::prelude::*;
use leptos::context::Provider;
use leptos::prelude::*;
use std::sync::Arc;
use time::OffsetDateTime;
use uuid::Uuid;

/// Shared runtime state for one mounted [`CrudInstance`].
///
/// [`CrudInstance`] provides this value to descendant components through Leptos's `provide_ontext`.
/// Copies refer to the same arena-owned signals and stored values. A copy is never immediately an
/// independent state snapshot.
///
/// Public mutation methods preserve the instance invariants. Signal setters remain private so
/// callers cannot bypass those methods.
#[derive(Debug, Clone, Copy)]
pub struct CrudInstanceContext {
    /// Volatile identifier for this mount.
    pub id: Uuid,

    /// Stable instance name used by [`CrudInstanceMgrContext`] lookups.
    /// Provided by the user. Required to be unique across the entire application.
    pub name: &'static str,

    default_config: StoredValue<CrudMutableInstanceConfig>,
    pub(crate) static_config: StoredValue<CrudStaticInstanceConfig>,
    pub(crate) data_provider: Signal<DynCrudRestDataProvider>,
    pub(crate) headers: ReadSignal<Vec<Header>>,
    pub(crate) create_elements: ReadSignal<CreateElements>,
    pub(crate) update_elements: ReadSignal<UpdateElements>,

    /// Navigation used by this mounted instance.
    pub navigation: CrudNavigation,

    /// The page the user is currently on in the list view.
    pub current_page: ReadSignal<PageNr>,
    set_current_page: WriteSignal<PageNr>,

    /// The amount of items shown per page in the list view.
    pub items_per_page: ReadSignal<ItemsPerPage>,
    set_items_per_page: WriteSignal<ItemsPerPage>,

    /// How data should be ordered when querying data for the ist view.
    pub order_by: ReadSignal<IndexMap<DynReadField, Order>>,
    set_order_by: WriteSignal<IndexMap<DynReadField, Order>>,

    /// Parent-resource configuration, when this instance is nested below another resource.
    pub parent: StoredValue<Option<CrudParentConfig>>,

    /// Current parent entity ID when parent-resource scoping can resolve one.
    pub parent_id: Signal<Option<SerializableId>>,

    /// Condition restricting this resource to the resolved parent entity.
    pub parent_id_referencing_condition: Signal<Option<Condition>>,

    /// The base condition applicable when fetching data.
    pub base_condition: Signal<Option<Condition>>,

    /// Entity awaiting confirmation for a single-entity deletion.
    pub deletion_request: ReadSignal<Option<DynReadOrUpdateModel>>,
    set_deletion_request: WriteSignal<Option<DynReadOrUpdateModel>>,
    set_deletion_navigation: WriteSignal<Option<CrudNavigation>>,

    /// Entities awaiting confirmation for a mass deletion.
    pub mass_deletion_request: ReadSignal<Option<Arc<Vec<DynReadModel>>>>,
    set_mass_deletion_request: WriteSignal<Option<Arc<Vec<DynReadModel>>>>,

    /// Token changed by [`Self::reload`] to refresh server-provided data.
    pub reload: ReadSignal<Uuid>,
    set_reload: WriteSignal<Uuid>,

    create_actions: ReadSignal<Option<CrudCreateActions>>,
    set_create_actions: WriteSignal<Option<CrudCreateActions>>,
}

impl CrudInstanceContext {
    /// Selects the list page to load.
    pub fn set_page(&self, page_number: PageNr) {
        self.set_current_page.set(page_number);
    }

    /// Sets the number of entities shown on each list page.
    pub fn set_items_per_page(&self, items_per_page: ItemsPerPage) {
        self.set_items_per_page.set(items_per_page);
    }

    // TODO: Why is this here and CrudInstanceConfig#update_order_by exists?
    /// Applies an ordering interaction for `field`.
    ///
    /// The interaction toggles the field between ascending and descending. It
    /// clears existing ordering first unless [`OrderByUpdateOptions::append`]
    /// is set.
    pub fn oder_by(&self, field: DynReadField, options: OrderByUpdateOptions) {
        self.set_order_by
            .update(|order_by: &mut IndexMap<DynReadField, Order>| {
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

    /// Records that the tab identified by `tab_id` was selected.
    ///
    /// The current implementation emits a diagnostic and retains no tab state.
    pub fn tab_selected(&self, tab_id: TabId) {
        tracing::info!(?tab_id, "tab_selected");
    }

    /// Opens single-entity deletion confirmation for `entity`.
    pub fn request_deletion_of(&self, entity: DynReadOrUpdateModel) {
        self.request_deletion_of_from(entity, self.navigation);
    }

    pub(crate) fn request_deletion_of_from(
        &self,
        entity: DynReadOrUpdateModel,
        navigation: CrudNavigation,
    ) {
        // TODO: Use upcasting instead of helper function when Rust 1.86 lands. (see: dyn upcasting coercion")
        self.set_deletion_navigation.set(Some(navigation));
        self.set_deletion_request.set(Some(entity));
    }

    /// Opens mass-deletion confirmation for `entities` when the collection is non-empty.
    pub fn request_mass_deletion(&self, entities: Arc<Vec<DynReadModel>>) {
        if !entities.is_empty() {
            self.set_mass_deletion_request.set(Some(entities));
        }
    }

    /// Cancels the current mass-deletion confirmation.
    pub fn cancel_mass_deletion(&self) {
        self.set_mass_deletion_request.set(None);
    }

    // TODO: Other functions do not take a . Should the instance provide its  to store it in this context? Would allow everyone to have access.
    /// Applies the shared UI effects from an action result.
    ///
    /// Both success and failure aftermaths may publish a toast or reload the
    /// instance. The `Result` variant records the action outcome; the contained
    /// [`CrudActionAftermath`] defines the UI effects in either case.
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

    /// Changes the reload token so data-dependent views fetch current server state.
    pub fn reload(&self) {
        self.set_reload.set(Uuid::new_v4());
    }

    pub(crate) fn create_actions(&self) -> Option<CrudCreateActions> {
        self.create_actions.get()
    }

    pub(crate) fn set_create_actions(&self, actions: Option<CrudCreateActions>) {
        self.set_create_actions.set(actions);
    }

    /// Reset this instance to its default configuration.
    /// Every change made by the user is reverted.
    pub fn reset(&self) {
        let context = *self;
        self.navigation
            .attempt(move || context.reset_committed(), || {});
    }

    fn reset_committed(&self) {
        let default = self.default_config.get_value();
        self.set_deletion_request.set(None);
        self.set_deletion_navigation.set(None);
        self.set_mass_deletion_request.set(None);
        self.set_current_page.set(default.page);
        self.set_items_per_page.set(default.items_per_page);
        self.set_order_by.set(default.order_by.clone());
        self.set_create_actions.set(None);
        self.navigation
            .navigate_committed(default.initial_view.clone());
    }
}

// TODO: Effect::new over all signals in config, bundle, serialize and store...

/// Mounts one configured CrudKit resource and renders its current registered view.
///
/// Without `navigation`, the instance creates navigation in a child of the
/// nearest manager navigation scope, initialized from
/// [`CrudInstanceConfig::initial_view`]. Supplied navigation remains
/// caller-owned and deliberately overrides the manager navigation scope. In
/// both cases, the instance mounts a private child navigation scope so
/// unmounting does not invalidate supplied navigation or remove the manager
/// navigation scope.
#[component]
pub fn CrudInstance(
    /// Stable name used to register this instance with its manager.
    name: &'static str,
    /// Resource, view, renderer, control, and request configuration for this mount.
    config: CrudInstanceConfig,
    /// Optional parent-resource relationship used to scope child data.
    #[prop(optional)]
    parent: Option<CrudParentConfig>,
    /// Optional caller-owned navigation that replaces manager-created navigation.
    #[prop(optional)]
    navigation: Option<CrudNavigation>,
    /// Optional callback invoked once after the instance context is created.
    #[prop(optional)]
    on_context_created: Option<Callback<CrudInstanceContext>>,
) -> impl IntoView {
    // Unique id of this instance. Volatile. Not persistent between rerenders.
    let id = Uuid::new_v4();

    let (config, static_config) = config.split();

    let static_config = StoredValue::new(static_config);

    let (api_base_url, _set_api_base_url) = signal(config.api_base_url.clone());
    let mgr = expect_context::<CrudInstanceMgrContext>();
    let instance_navigation =
        navigation.unwrap_or_else(|| mgr.child_navigation(config.initial_view.clone()));
    let view = instance_navigation.current();
    let navigation = instance_navigation.scoped();
    navigation.register_confirmation_host();

    mgr.register(name, InstanceState { name, view });

    let (headers, _set_headers) = signal(config.headers.clone());
    let (current_page, set_current_page) = signal(config.page);
    let (items_per_page, set_items_per_page) = signal(config.items_per_page);
    let (order_by, set_order_by) = signal(config.order_by.clone());

    let parent = StoredValue::new(parent);
    let parent_id = Signal::derive(move || {
        parent
            .read_value()
            .as_ref()
            .and_then(|parent| get_parent_id(parent, mgr))
    });
    let parent_id_referencing_condition = Signal::derive(move || {
        parent
            .read_value()
            .as_ref()
            .and_then(|parent| get_parent_id(parent, mgr).map(|id| (parent, id)))
            .and_then(|(parent, id)| {
                let entry = id
                    .into_entries()
                    .find(|entry| entry.field_name == parent.referenced_field);

                let Some(SerializableIdEntry {
                    field_name: _,
                    value,
                }) = entry
                else {
                    tracing::warn!(
                        referenced_field = %parent.referenced_field,
                        "Referenced field not found in parent ID"
                    );
                    return None;
                };

                let Ok(clause_value) = value.clone().try_into() else {
                    tracing::warn!("Parent ID value not convertible to condition clause value");
                    return None;
                };

                Some(Condition::All(vec![ConditionElement::Clause(
                    ConditionClause {
                        column_name: parent.referencing_field.to_string(),
                        operator: crudkit_core::condition::Operator::Equal,
                        value: clause_value,
                    },
                )]))
            })
    });
    let base_condition = config.base_condition.clone();
    let base_condition = Signal::derive(move || {
        crudkit_core::condition::merge_conditions(
            parent_id_referencing_condition.get(),
            base_condition.clone(),
        )
    });
    let (create_elements, _set_create_elements) = signal(config.create_elements.clone());
    let (update_elements, _set_update_elements) = signal(config.elements.clone());
    let (deletion_request, set_deletion_request) = signal(None);
    let (deletion_navigation, set_deletion_navigation) = signal(None::<CrudNavigation>);
    let (mass_deletion_request, set_mass_deletion_request) = signal(None::<Arc<Vec<DynReadModel>>>);
    let (reload, set_reload) = signal(Uuid::new_v4());
    let (create_actions, set_create_actions) = signal(None::<CrudCreateActions>);

    let default_config = StoredValue::new(config);

    let data_provider = Signal::derive(move || {
        DynCrudRestDataProvider::new(
            api_base_url.get(),
            static_config.read_value().reqwest_executor.clone(),
            static_config.read_value().resource_name.clone(),
        )
    });

    let ctx = CrudInstanceContext {
        id,
        name,
        default_config,
        static_config,
        data_provider,
        headers,
        create_elements,
        update_elements,
        navigation,
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
        set_deletion_navigation,
        mass_deletion_request,
        set_mass_deletion_request,
        reload,
        set_reload,
        create_actions,
        set_create_actions,
    };
    if let Some(on_context_created) = on_context_created {
        on_context_created.run(ctx)
    }

    let on_cancel_delete = Callback::new(move |()| {
        tracing::info!("Removing delete request");
        set_deletion_request.set(None);
        set_deletion_navigation.set(None);
    });

    let delete_action = Action::new_local(
        move |(entity_id, deletion_navigation): &(SerializableId, CrudNavigation)| {
            let data_provider = data_provider.get();
            let id = entity_id.clone();
            let deletion_navigation = *deletion_navigation;
            async move {
                let result = data_provider.delete_by_id(DeleteById { id }).await;
                let deleted = result.is_ok();

                // The delete operation was performed and must therefore no longer be requested.
                set_deletion_request.set(None);
                set_deletion_navigation.set(None);

                // The user must be notified how the delete operation went.
                handle_delete_result(result);

                if deleted {
                    // Persistence resolved the edited draft, so follow-up
                    // navigation must not ask about the now-committed input again.
                    if deletion_navigation.return_committed() == CommittedReturnDestination::View {
                        ctx.reload();
                    }
                }
            }
        },
    );

    let on_accept_delete = Callback::new(move |entity: DynReadOrUpdateModel| {
        let id = match entity {
            DynReadOrUpdateModel::Read(model) => model.id(),
            DynReadOrUpdateModel::Update(model) => model.id(),
        };
        let Some(deletion_navigation) = deletion_navigation.get_untracked() else {
            tracing::error!("Delete request had no originating navigation scope");
            set_deletion_request.set(None);
            return;
        };
        delete_action.dispatch((id, deletion_navigation));
    });

    let on_cancel_delete_many = Callback::new(move |()| {
        tracing::info!("Removing mass delete request");
        set_mass_deletion_request.set(None);
    });

    let delete_many_action = Action::new_local(move |entities: &Arc<Vec<DynReadModel>>| {
        let data_provider = data_provider.get();
        let entities = entities.clone();
        async move {
            // Build a condition from all selected entity IDs.
            // Each entity's ID fields become an AND condition, and all entities are OR'd together.
            let condition = build_condition_from_entities(&entities);

            let result = data_provider
                .delete_many(DynDeleteMany {
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

    let on_accept_delete_many = Callback::new(move |entities: Arc<Vec<DynReadModel>>| {
        delete_many_action.dispatch(entities);
    });

    let view_registry = static_config.read_value().view_registry.clone();
    let pending_navigation_requiring_leave_confirmation = navigation.requires_leave_confirmation();

    view! {
        <Provider value=ctx>
            <div class="crud-instance">
                <div class="body">
                    {move || {
                        view! {
                            <RegisteredCrudView
                                context=ctx
                                registry=view_registry.clone()
                                view=view.get()
                                navigation=navigation
                            />
                        }
                    }}
                    <CrudDeleteModal
                        entity=deletion_request
                        on_cancel=on_cancel_delete
                        on_accept=on_accept_delete
                    />
                    <CrudDeleteManyModal
                        entities=mass_deletion_request
                        on_cancel=on_cancel_delete_many
                        on_accept=on_accept_delete_many
                    />
                    <CrudLeaveModal
                        show_when=pending_navigation_requiring_leave_confirmation
                        on_cancel=move || navigation.cancel_pending()
                        on_accept=move || navigation.accept_pending()
                    />
                </div>
            </div>
        </Provider>
    }
}

#[component]
fn RegisteredCrudView(
    context: CrudInstanceContext,
    registry: CrudViewRegistry,
    view: CrudView,
    navigation: CrudNavigation,
) -> impl IntoView {
    let navigation = navigation.scoped();
    provide_context(CrudInstanceContext {
        navigation,
        ..context
    });
    registry.render(view, navigation)
}

fn get_parent_id(parent: &CrudParentConfig, mgr: CrudInstanceMgrContext) -> Option<SerializableId> {
    // Uses untracked access via StoredValue::read_value() internally.
    // Otherwise, at instance nesting depth 3, rendering the instance and registering it would
    // cause instance at depth 2 to register this change here and force a field-rerender.
    let parent_state = mgr.get_by_name(parent.name)?;
    parent_state.view.get_untracked().subject
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
fn build_condition_from_entities(entities: &[DynReadModel]) -> Condition {
    use crudkit_core::condition::TryIntoAllEqualCondition;

    if entities.is_empty() {
        return Condition::none();
    }

    let entity_conditions: Vec<Condition> = entities
        .iter()
        .filter_map(|entity| {
            let id = entity.id();
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
