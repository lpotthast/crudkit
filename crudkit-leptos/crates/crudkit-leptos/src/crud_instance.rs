use std::marker::PhantomData;

use crudkit_id::{Id, SerializableId};
use crudkit_shared::{DeleteResult, Order};
use crudkit_web::{prelude::*, TabId};
use indexmap::IndexMap;
use leptonic::components::prelude::*;
use leptos::*;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    crud_action::CrudActionAftermath,
    crud_delete_modal::CrudDeleteModal,
    crud_instance_config::{CrudInstanceConfig, CrudParentConfig, CrudStaticInstanceConfig},
    crud_instance_mgr::InstanceState,
    crud_read_view::CrudReadView,
    prelude::{CrudCreateView, CrudEditView, CrudInstanceMgrContext, CrudListView},
};

/// Runtime data of this instance, provided to child components through provide_context.
///
/// This context struct contains data not really necessary in every view,
/// but as we want to retain all state between view changes, this is a reasonable place to store that state.
/// It allows a user to configure the list view, update an entry, return, and then find the list view unaltered.
///¶
/// Signal setters should generally not be pub. Define custom functions providing the required functionality.
#[derive(Debug, Clone)]
pub struct CrudInstanceContext<T: CrudMainTrait + 'static> {
    default_config: StoredValue<CrudInstanceConfig<T>>,

    /// The current "view" of this instance. Can be List, Create, Edit, Read, ... Acts like a router...
    pub view: ReadSignal<CrudView<T::ReadModelId, T::UpdateModelId>>,
    set_view: WriteSignal<CrudView<T::ReadModelId, T::UpdateModelId>>,

    /// The page the user is currently on in the list view.
    pub current_page: ReadSignal<u64>,
    set_current_page: WriteSignal<u64>,

    /// The amount of items shown per page in the list view.
    pub items_per_page: ReadSignal<u64>,
    set_items_per_page: WriteSignal<u64>,

    /// How data should be ordered when querying data for the ist view.
    pub order_by: ReadSignal<IndexMap<<T::ReadModel as CrudDataTrait>::Field, Order>>,
    set_order_by: WriteSignal<IndexMap<<T::ReadModel as CrudDataTrait>::Field, Order>>,

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
    pub deletion_request: ReadSignal<Option<DeletableModel<T::ReadModel, T::UpdateModel>>>,
    set_deletion_request: WriteSignal<Option<DeletableModel<T::ReadModel, T::UpdateModel>>>,

    /// Whenever this signal changes, the current view should "refresh" by reloading all server provided data.
    /// It simply provides a new random ID on each invocation.
    pub reload: ReadSignal<Uuid>,
    set_reload: WriteSignal<Uuid>,
}

// TODO: This allows us to easily use this context but more copies are made...
impl<T: CrudMainTrait> Copy for CrudInstanceContext<T> {}

impl<T: CrudMainTrait + 'static> CrudInstanceContext<T> {
    /// Opens the list view.
    pub fn list(&self) {
        self.set_view.update(|view| *view = CrudView::List);
    }

    /// Opens the create view.
    pub fn create(&self) {
        self.set_view.update(|view| *view = CrudView::Create);
    }

    /// Opens the read view for the given entity.
    pub fn read(&self, entity_with_id: T::ReadModelId) {
        self.set_view
            .update(|view| *view = CrudView::Read(entity_with_id));
    }

    /// Opens the edit view for the given entity.
    pub fn edit(&self, entity_with_id: T::UpdateModelId) {
        self.set_view
            .update(|view| *view = CrudView::Edit(entity_with_id));
    }

    pub fn set_page(&self, page_number: u64) {
        self.set_current_page.set(page_number);
    }

    pub fn set_items_per_page(&self, items_per_page: u64) {
        self.set_items_per_page.set(items_per_page);
    }

    pub fn oder_by(
        &self,
        field: <T::ReadModel as CrudDataTrait>::Field,
        options: OrderByUpdateOptions,
    ) {
        self.set_order_by.update(
            |order_by: &mut IndexMap<
                <<T as CrudMainTrait>::ReadModel as CrudDataTrait>::Field,
                Order,
            >| {
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
            },
        )
    }

    pub fn tab_selected(&self, tab_id: TabId) {
        tracing::info!(?tab_id, "tab_selected");
    }

    pub fn request_deletion_of(&self, entity: DeletableModel<T::ReadModel, T::UpdateModel>) {
        self.set_deletion_request.set(Some(entity));
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
        // TODO: Should there be functions resetting individual views? This always resets everything and sets the view to be the List view...
        self.set_view.set(default.view);
        self.set_current_page.set(default.page);
        self.set_items_per_page.set(default.items_per_page);
        self.set_order_by.set(default.order_by.clone());
        self.set_deletion_request.set(None);
    }
}

// TODO: create_effect over all signals in config, bundle, serialize and store...

#[component]
pub fn CrudInstance<T>(
    // TODO: Analyze children once on creation and on prop changes. Pass generated data-structure to children!
    // TODO: Only allow easy-to-parse structure:
    /*
       tbd...
       ListDetails {

       }
       FieldDetails {

       }
    */
    //#[prop_or_default]
    //pub children: ChildrenRenderer<Item>,
    name: &'static str,

    //#[prop(into)] api_base_url: Signal<String>,
    //#[prop(into)] view: Signal<CrudView<T::ReadModelId, T::UpdateModelId>>,
    //#[prop(into)] headers: Signal<Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions)>>,
    //#[prop(into)] create_elements: Signal<CreateElements<T>>,
    //#[prop(into)] elements: Signal<Vec<Elem<T::UpdateModel>>>,
    //#[prop(into)] order_by: Signal<IndexMap<<T::ReadModel as CrudDataTrait>::Field, Order>>,
    //#[prop(into)] items_per_page: Signal<u64>,
    //#[prop(into)] current_page: Signal<u64>,
    //#[prop(into)] active_tab: Signal<Option<Label>>,
    //#[prop(into)] nested: Signal<Option<NestedConfig>>,
    config: CrudInstanceConfig<T>,
    static_config: CrudStaticInstanceConfig<T>,
    #[prop(optional)] parent: Option<CrudParentConfig>,
    #[prop(optional)] on_context_created: Option<Callback<CrudInstanceContext<T>>>,
    //pub portal_target: Option<String>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let (api_base_url, set_api_base_url) = create_signal(config.api_base_url.clone());
    let (view, set_view) = create_signal(config.view.clone());
    // TODO: As memo?
    let serializable_view = Signal::<SerializableCrudView>::derive(move || view.get().into());

    let mgr = expect_context::<CrudInstanceMgrContext>();
    mgr.register(
        name,
        InstanceState {
            name,
            view: serializable_view,
        },
    );

    let (headers, set_headers) = create_signal(config.headers.clone());
    let (current_page, set_current_page) = create_signal(config.page.clone());
    let (items_per_page, set_items_per_page) = create_signal(config.items_per_page.clone());
    let (order_by, set_order_by) = create_signal(config.order_by.clone());

    fn get_parent_id(
        parent: &CrudParentConfig,
        mgr: CrudInstanceMgrContext,
    ) -> Option<SerializableId> {
        let parent_state = mgr
            .instances
            .get()
            .get_by_name(parent.name)
            .expect("parent to be managed");
        match parent_state.view.get() {
            SerializableCrudView::List => None,
            SerializableCrudView::Create => None,
            SerializableCrudView::Read(id) => Some(id),
            SerializableCrudView::Edit(id) => Some(id),
        }
    }

    let parent = store_value(parent);
    let parent_id = Signal::derive(move || {
        parent
            .get_value()
            .and_then(|parent| get_parent_id(&parent, mgr))
    });
    let parent_id_referencing_condition = Signal::derive(move || {
        parent
            .get_value()
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
                            value: value.clone().into(),
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
    let (create_elements, set_create_elements) = create_signal(config.create_elements.clone());
    let (update_elements, set_update_elements) = create_signal(config.elements.clone());
    let (deletion_request, set_deletion_request) = create_signal(None);
    let (reload, set_reload) = create_signal(Uuid::new_v4());

    let default_config = store_value(config);

    let data_provider = create_memo(move |_| CrudRestDataProvider::<T>::new(api_base_url.get()));

    // ctx is copy. But is it efficient? Do we want to put this into a stored value instead?
    let ctx = CrudInstanceContext::<T> {
        default_config,
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
        reload,
        set_reload,
    };
    provide_context(ctx);
    if let Some(on_context_created) = on_context_created {
        on_context_created.call(ctx)
    }

    let custom_read_fields = Signal::derive(move || static_config.custom_read_fields.clone());
    let custom_create_fields = Signal::derive(move || static_config.custom_create_fields.clone());
    let custom_update_fields = Signal::derive(move || static_config.custom_update_fields.clone());

    let create_field_config =
        Signal::derive(move || static_config.create_field_select_config.clone());
    let read_field_config = Signal::derive(move || static_config.read_field_select_config.clone());
    let update_field_config =
        Signal::derive(move || static_config.update_field_select_config.clone());

    let actions = Signal::derive(move || static_config.actions.clone());
    let entity_actions = Signal::derive(move || static_config.entity_actions.clone());

    let on_cancel_delete = Callback::new(move |()| {
        tracing::info!("Removing delete request");
        set_deletion_request.set(None);
    });

    // TODO: Always open the list view after a successful delete.
    let on_accept_delete = Callback::new(
        move |entity: DeletableModel<T::ReadModel, T::UpdateModel>| {
            // TODO: A create_action_once could save us a clone...
            let action = create_action(move |_data: &()| {
                let id = match &entity {
                    DeletableModel::Read(entity) => entity.get_id().into_serializable_id(),
                    DeletableModel::Update(entity) => entity.get_id().into_serializable_id(),
                };
                let data_provider = data_provider.get();
                async move {
                    data_provider
                        .delete_by_id(DeleteById { id: id.clone() })
                        .await
                }
            });
            action.dispatch(());

            let value = action.value();
            create_effect(move |_prev| {
                if let Some(result) = value.get() {
                    // The delete operation was performed and must therefore no longer be requested.
                    set_deletion_request.set(None);

                    // No matter where the user deleted an entity, the list view should be shown afterwards.
                    ctx.list();

                    // The user must be notified how the delete operation went.
                    match result {
                        Ok(delete_result) => {
                            match delete_result {
                                DeleteResult::Deleted(num) => {
                                    expect_context::<Toasts>().push(Toast {
                                        id: Uuid::new_v4(),
                                        created_at: OffsetDateTime::now_utc(),
                                        variant: ToastVariant::Success,
                                        header: "Löschen".into_view(),
                                        body: format!(
                                            "{num} {} erfolgreich gelöscht.",
                                            match num {
                                                1 => "Eintrag",
                                                _ => "Einträge",
                                            }
                                        )
                                        .into_view(),
                                        timeout: ToastTimeout::DefaultDelay,
                                    })
                                }

                                DeleteResult::Aborted { reason } => expect_context::<Toasts>()
                                    .push(Toast {
                                        id: Uuid::new_v4(),
                                        created_at: OffsetDateTime::now_utc(),
                                        variant: ToastVariant::Warn,
                                        header: "Delete".into_view(),
                                        body: format!("Löschvorgang abgebrochen. Grund: {reason}")
                                            .into_view(),
                                        timeout: ToastTimeout::DefaultDelay,
                                    }),
                                DeleteResult::CriticalValidationErrors => {
                                    expect_context::<Toasts>().push(Toast {
                                        id: Uuid::new_v4(),
                                        created_at: OffsetDateTime::now_utc(),
                                        variant: ToastVariant::Error,
                                        header: "Delete".into_view(),
                                        body: format!("{delete_result:#?}").into_view(),
                                        timeout: ToastTimeout::DefaultDelay,
                                    })
                                }
                            }
                            ctx.reload()
                        }
                        Err(err) => {
                            expect_context::<Toasts>().push(Toast {
                                id: Uuid::new_v4(),
                                created_at: OffsetDateTime::now_utc(),
                                variant: ToastVariant::Error,
                                header: "Delete".into_view(),
                                body: format!("Konnte Eintrag nicht Löschen: {err:#?}").into_view(),
                                timeout: ToastTimeout::DefaultDelay,
                            });
                            ctx.reload()
                        }
                    }
                }
            });
        },
    );

    view! {
        <div class="crud-instance">
            <div class="body">
                {move || match view.get() {
                    CrudView::List => {
                        view! {
                            <CrudListView
                                api_base_url=api_base_url
                                data_provider=data_provider
                                headers=headers
                                order_by=order_by
                                custom_fields=custom_read_fields
                                field_config=read_field_config
                                actions=actions
                            />
                        }.into_view()
                    }
                    CrudView::Create => {
                        view! {
                            <CrudCreateView
                                _phantom={PhantomData::<T>::default()}
                                api_base_url=api_base_url
                                data_provider=data_provider
                                create_elements=create_elements
                                custom_fields=custom_create_fields
                                field_config=create_field_config
                                on_edit_view=move |id| ctx.edit(id)
                                on_list_view=move |()| ctx.list()
                                on_create_view=move |()| ctx.create()
                                on_entity_created=move |saved| {}
                                on_entity_creation_aborted=move |reason| {}
                                on_entity_not_created_critical_errors=move |()| {}
                                on_entity_creation_failed=move |request_error| {}
                                on_tab_selected=move |tab_id| {
                                    ctx.tab_selected(tab_id)
                                }
                            />
                        }.into_view()
                    }
                    CrudView::Read(id) => {
                        view! {
                            <CrudReadView
                                _phantom={PhantomData::<T>::default()}
                                api_base_url=api_base_url
                                // TODO: This cant be good...
                                id=Signal::derive(move || id.clone())
                                data_provider=data_provider
                                actions=entity_actions
                                elements=update_elements
                                custom_fields=custom_update_fields
                                field_config=update_field_config
                                on_list_view=move |()| ctx.list()
                                on_tab_selected=move |tab_id| {
                                    ctx.tab_selected(tab_id)
                                }
                            />
                        }.into_view()
                    }
                    CrudView::Edit(id) => {
                        view! {
                            // TODO: This cant be good...
                            <CrudEditView
                                _phantom={PhantomData::<T>::default()}
                                api_base_url=api_base_url
                                // TODO: This cant be good...
                                id=Signal::derive(move || id.clone())
                                data_provider=data_provider
                                actions=entity_actions
                                elements=update_elements
                                custom_fields=custom_update_fields
                                field_config=update_field_config
                                on_list_view=move |()| ctx.list()
                                on_create_view=move |()| ctx.create()
                                on_entity_updated=move |saved| {}
                                on_entity_update_aborted=move |reason| {}
                                on_entity_not_updated_critical_errors=move |()| {}
                                on_entity_update_failed=move |request_error| {}
                                on_tab_selected=move |tab_id| {
                                    ctx.tab_selected(tab_id)
                                }
                            />
                        }.into_view()
                    }
                }}
                <CrudDeleteModal
                    _phantom={PhantomData::<T>::default()}
                    entity=deletion_request
                    on_cancel=on_cancel_delete.clone()
                    on_accept=on_accept_delete.clone()
                />
            </div>
        </div>
    }
}
