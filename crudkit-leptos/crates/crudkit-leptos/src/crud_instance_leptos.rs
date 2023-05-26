use std::cell::RefCell;

use crudkit_id::Id;
use crudkit_shared::{DeleteResult, Order};
use crudkit_web::prelude::*;
use indexmap::IndexMap;
use leptonic::toast::{Toast, ToastTimeout, ToastVariant, Toasts};
use leptos::*;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    crud_action::CrudActionAftermath,
    crud_delete_modal::CrudDeleteModal,
    crud_instance_config::{
        CreateElements, CrudInstanceConfig, CrudStaticInstanceConfig, NestedConfig,
    },
    prelude::CrudListViewL,
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

    /// Whenever the user requests to delete something, this is the place that information is stored.
    pub deletion_request: ReadSignal<Option<DeletableModel<T::ReadModel, T::UpdateModel>>>,
    set_deletion_request: WriteSignal<Option<DeletableModel<T::ReadModel, T::UpdateModel>>>,

    /// Whenever this signal changes, the current view should "refresh" by reloading all server provided data.
    /// It simply provides a new random ID on each invocation.
    pub reload: ReadSignal<Uuid>,
    set_reload: WriteSignal<Uuid>,
}

impl<T: CrudMainTrait + 'static> CrudInstanceContext<T> {
    /// Opens the create view.
    pub fn create(&self) {
        self.set_view.update(|view| *view = CrudView::Create);
    }

    /// Opens the read view for the given entity.
    pub fn read(&self, entity: T::ReadModel) {
        self.set_view
            .update(|view| *view = CrudView::Read(entity.get_id()));
    }

    /// Opens the edit view for the given entity.
    pub fn edit(&self, entity: T::UpdateModel) {
        self.set_view
            .update(|view| *view = CrudView::Edit(entity.get_id()));
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
                //tracing::debug!(?field, ?options, "order by");
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

    pub fn request_deletion_of(&self, entity: DeletableModel<T::ReadModel, T::UpdateModel>) {
        self.set_deletion_request.set(Some(entity));
    }

    // TODO: Other functions do not take a Scope. Should the instance provide its scope to store it in this context? Would allow everyone to have access.
    pub fn handle_action_outcome(
        &self,
        cx: Scope,
        outcome: Result<CrudActionAftermath, CrudActionAftermath>,
    ) {
        tracing::info!(?outcome, "handling action outcome");

        let CrudActionAftermath {
            show_toast,
            reload_data,
        } = match outcome {
            Ok(outcome) => outcome,
            Err(outcome) => outcome,
        };

        if let Some(toast) = show_toast {
            expect_context::<Toasts>(cx).push(toast);
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
    cx: Scope,

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
    #[prop(into)] name: String,

    #[prop(into)] api_base_url: Signal<String>,
    #[prop(into)] view: Signal<CrudView<T::ReadModelId, T::UpdateModelId>>,
    #[prop(into)] headers: Signal<Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions)>>,
    #[prop(into)] create_elements: Signal<CreateElements<T>>,
    #[prop(into)] elements: Signal<Vec<Elem<T::UpdateModel>>>,
    #[prop(into)] order_by: Signal<IndexMap<<T::ReadModel as CrudDataTrait>::Field, Order>>,
    #[prop(into)] items_per_page: Signal<u64>,
    #[prop(into)] current_page: Signal<u64>,
    #[prop(into)] active_tab: Signal<Option<Label>>,
    #[prop(into)] nested: Signal<Option<NestedConfig>>,

    config: CrudInstanceConfig<T>,
    static_config: CrudStaticInstanceConfig<T>,
    //pub portal_target: Option<String>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let (view, set_view) = create_signal(cx, view.get());
    let (current_page, set_current_page) = create_signal(cx, current_page.get());
    let (items_per_page, set_items_per_page) = create_signal(cx, items_per_page.get());
    let (order_by, set_order_by) = create_signal(cx, order_by.get());
    let (deletion_request, set_deletion_request) = create_signal(cx, None);
    let show_delete_modal = Signal::derive(cx, move || deletion_request.get().is_some());
    let (reload, set_reload) = create_signal(cx, Uuid::new_v4());

    let data_provider = Signal::derive(cx, move || {
        CrudRestDataProvider::<T>::new(api_base_url.get())
    });

    let default_config = store_value(cx, config);

    provide_context(
        cx,
        CrudInstanceContext::<T> {
            default_config,
            view,
            set_view,
            current_page,
            set_current_page,
            items_per_page,
            set_items_per_page,
            order_by,
            set_order_by,
            deletion_request,
            set_deletion_request,
            reload,
            set_reload,
        },
    );

    let custom_read_fields = Signal::derive(cx, move || static_config.custom_read_fields.clone());

    let actions = Signal::derive(cx, move || static_config.actions.clone());
    let entity_actions = Signal::derive(cx, move || static_config.entity_actions.clone());

    let on_cancel_delete = move || {
        tracing::info!("Removing delete request");
        set_deletion_request.set(None);
    };
    let on_accept_delete = move |entity: DeletableModel<T::ReadModel, T::UpdateModel>| {
        // TODO: A create_action_once could save us a clone...
        let action = create_action(cx, move |_data: &()| {
            let id = match &entity {
                DeletableModel::Read(entity) => entity.get_id().into_serializable_id(),
                DeletableModel::Update(entity) => entity.get_id().into_serializable_id(),
            };
            async move {
                data_provider
                    .get()
                    .delete_by_id(DeleteById { id: id.clone() })
                    .await
            }
        });
        action.dispatch(());

        let value = action.value();
        create_effect(cx, move |_prev| {
            if let Some(result) = value.get() {
                // The delete operation was performed and must therefore no longer be requested.
                set_deletion_request.set(None);

                // The user must be notified how the delete operation went.
                match result {
                    Ok(delete_result) => {
                        match delete_result {
                            DeleteResult::Deleted(num) => {
                                expect_context::<Toasts>(cx).push(Toast {
                                    id: Uuid::new_v4(),
                                    created_at: OffsetDateTime::now_utc(),
                                    variant: ToastVariant::Success,
                                    header: view! {cx, "Löschen" }.into_view(cx),
                                    body: view! {cx, { format!("Erfolgreich {num} Einträge geköscht.") } }
                                        .into_view(cx),
                                    timeout: ToastTimeout::DefaultDelay,
                                })
                            }

                            DeleteResult::Aborted { reason } => {
                                expect_context::<Toasts>(cx).push(Toast {
                                    id: Uuid::new_v4(),
                                    created_at: OffsetDateTime::now_utc(),
                                    variant: ToastVariant::Warn,
                                    header: view! {cx, "Delete" }.into_view(cx),
                                    body: view! {cx, { format!("Löschvorgang abgebrochen. Grund: {reason}") } }
                                        .into_view(cx),
                                    timeout: ToastTimeout::DefaultDelay,
                                })
                            }
                            DeleteResult::CriticalValidationErrors => expect_context::<Toasts>(cx)
                                .push(Toast {
                                    id: Uuid::new_v4(),
                                    created_at: OffsetDateTime::now_utc(),
                                    variant: ToastVariant::Error,
                                    header: view! {cx, "Delete" }.into_view(cx),
                                    body: view! {cx, { format!("{delete_result:#?}") } }
                                        .into_view(cx),
                                    timeout: ToastTimeout::DefaultDelay,
                                }),
                        }
                        expect_context::<CrudInstanceContext<T>>(cx).reload()
                    }
                    Err(err) => {
                        expect_context::<Toasts>(cx).push(Toast {
                            id: Uuid::new_v4(),
                            created_at: OffsetDateTime::now_utc(),
                            variant: ToastVariant::Error,
                            header: view! {cx, "Delete" }.into_view(cx),
                            body:
                                view! {cx, { format!("Konnte Eintrag nicht Löschen: {err:#?}") } }
                                    .into_view(cx),
                            timeout: ToastTimeout::DefaultDelay,
                        });
                        expect_context::<CrudInstanceContext<T>>(cx).reload()
                    }
                }
            }
        });
    };

    let content = move |cx, view| {
        view! {cx,
            <div class="crud-instance">
                <div class="body">
                    { match view {
                        CrudView::List => {
                            view! {cx,
                                <CrudListViewL
                                    api_base_url=api_base_url
                                    data_provider=data_provider
                                    headers=headers
                                    //children={ctx.props().children.clone()}
                                    custom_fields=custom_read_fields
                                    actions=actions
                                />
                            }
                            .into_view(cx)
                        }
                        CrudView::Create => {
                            view! {cx,
                                "create"
                                //<CrudCreateView<T>
                                //    data_provider={self.data_provider.clone()}
                                //    parent_id={self.parent_id.clone()}
                                //    children={ctx.props().children.clone()}
                                //    custom_create_fields={self.static_config.custom_create_fields.clone()}
                                //    custom_update_fields={self.static_config.custom_update_fields.clone()}
                                //    config={self.config.clone()}
                                //    list_view_available={true}
                                //    on_list_view={ctx.link().callback(|_| Msg::List)}
                                //    on_entity_created={ctx.link().callback(Msg::EntityCreated)}
                                //    on_entity_creation_aborted={ctx.link().callback(Msg::EntityCreationAborted)}
                                //    on_entity_not_created_critical_errors={ctx.link().callback(|_| Msg::EntityNotCreatedDueToCriticalErrors)}
                                //    on_entity_creation_failed={ctx.link().callback(Msg::EntityCreationFailed)}
                                //    on_link={ctx.link().callback(|link: Option<Scope<CrudCreateView<T>>>|
                                //        Msg::ViewLinked(link.map(|link| ViewLink::Create(link))))}
                                //    on_tab_selected={ctx.link().callback(|label| Msg::TabSelected(label))}
                                // />
                            }
                            .into_view(cx)
                        }
                        CrudView::Read(id) => {
                            view! {cx,
                                "read"
                                //<CrudReadView<T>
                                //    data_provider={self.data_provider.clone()}
                                //    children={ctx.props().children.clone()}
                                //    custom_fields={self.static_config.custom_update_fields.clone()}
                                //    config={self.config.clone()}
                                //    id={id.clone()}
                                //    list_view_available={true}
                                //    on_list_view={ctx.link().callback(|_| Msg::List)}
                                //    on_tab_selected={ctx.link().callback(|label| Msg::TabSelected(label))}
                                // />
                            }
                            .into_view(cx)
                        }
                        CrudView::Edit(id) => {
                            view! {cx,
                                "edit"
                                //<CrudEditView<T>
                                //    data_provider={self.data_provider.clone()}
                                //    children={ctx.props().children.clone()}
                                //    custom_fields={self.static_config.custom_update_fields.clone()}
                                //    config={self.config.clone()}
                                //    static_config={self.static_config.clone()}
                                //    id={id.clone()}
                                //    list_view_available={true}
                                //    on_entity_updated={ctx.link().callback(Msg::EntityUpdated)}
                                //    on_entity_update_aborted={ctx.link().callback(Msg::EntityUpdateAborted)}
                                //    on_entity_not_updated_critical_errors={ctx.link().callback(|_| Msg::EntityNotUpdatedDueToCriticalErrors)}
                                //    on_entity_update_failed={ctx.link().callback(Msg::EntityUpdateFailed)}
                                //    on_list={ctx.link().callback(|_| Msg::List)}
                                //    on_create={ctx.link().callback(|_| Msg::Create)}
                                //    on_delete={ctx.link().callback(|entity| Msg::Delete(DeletableModel::Update(entity)))}
                                //    on_link={ctx.link().callback(|link: Option<Scope<CrudEditView<T>>>|
                                //        Msg::ViewLinked(link.map(|link| ViewLink::Edit(link))))}
                                //    on_tab_selected={ctx.link().callback(|label| Msg::TabSelected(label))}
                                //    on_entity_action={ctx.link().callback(Msg::CustomEntityAction)}
                                // />
                            }
                            .into_view(cx)
                        }
                    } }

                    { move || match deletion_request.get() {
                        Some(deletable_model) => {
                            match deletable_model {
                                DeletableModel::Read(read_model) => view! {cx,
                                    <CrudDeleteModal
                                        show_when=show_delete_modal
                                        entity=read_model.clone()
                                        on_cancel=on_cancel_delete
                                        on_accept=move |entity| on_accept_delete(DeletableModel::Read(entity))>
                                    </CrudDeleteModal>
                                }.into_view(cx),
                                DeletableModel::Update(update_model) => view! {cx,
                                    <CrudDeleteModal
                                        show_when=show_delete_modal
                                        entity=update_model.clone()
                                        on_cancel=on_cancel_delete
                                        on_accept=move |entity| on_accept_delete(DeletableModel::Update(entity))>
                                    </CrudDeleteModal>
                                }.into_view(cx)
                            }
                        },
                        None => {
                            tracing::info!("Delete: None");
                            ().into_view(cx)
                        }
                    } }
                </div>
            </div>
        }
    };

    let child_scope = RefCell::new(Option::<Scope>::None);
    let router = move || {
        // Whenever the "view" changes, the new view must be rendered in a new scope.
        let view: CrudView<<T as CrudMainTrait>::ReadModelId, <T as CrudMainTrait>::UpdateModelId> =
            view.get();

        let (view, _) = cx.run_child_scope(|cx| {
            let prev_cx = std::mem::replace(&mut *child_scope.borrow_mut(), Some(cx));
            if let Some(prev_cx) = prev_cx {
                prev_cx.dispose();
            }

            content(cx, view).into_view(cx)
        });

        view
    };

    router
}
