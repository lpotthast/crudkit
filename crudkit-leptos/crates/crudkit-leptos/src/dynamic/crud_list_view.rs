use crate::dynamic::crud_action::{CrudAction, ResourceActionViewInput};
use crate::dynamic::crud_action_context::CrudActionContext;
use crate::dynamic::crud_instance::CrudInstanceContext;
use crate::dynamic::crud_instance_config::{FieldRendererRegistry, Header};
use crate::dynamic::crud_table::{CrudTable, NoDataAvailable};
use crate::shared::crud_pagination::CrudPagination;
use crudkit_core::Order;
use crudkit_web::dynamic::prelude::*;
use crudkit_web::dynamic::{AnyReadField, AnyReadModel, SerializableReadField};
use indexmap::IndexMap;
use leptonic::components::prelude::*;
use leptonic::prelude::*;
use leptos::prelude::*;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub struct CrudListViewContext {
    pub data: Memo<Result<Arc<Vec<AnyReadModel>>, NoDataAvailable>>,
    pub has_data: Signal<bool>,

    pub selected: ReadSignal<Arc<Vec<AnyReadModel>>>,
    set_selected: WriteSignal<Arc<Vec<AnyReadModel>>>,
    pub all_selected: Signal<bool>,
}

impl CrudListViewContext {
    pub fn clear_selection(&self) {
        self.set_selected
            .update(|selected| *selected = Arc::new(Vec::new()));
    }

    pub fn toggle_select_all(&self) {
        if let Ok(data) = self.data.get_untracked() {
            let selected = self.selected.get_untracked();
            if selected.len() < data.len() {
                // Select all
                self.set_selected
                    .update(|selected| *selected = data.clone());
            } else {
                // Deselect all
                self.clear_selection();
            }
        } else {
            tracing::warn!("Tried to toggle_select_all when no data was present.");
        }
    }

    pub fn select(&self, entity: AnyReadModel, state: bool) {
        self.set_selected.update(|list| {
            let mut selected = list.as_ref().clone();

            let pos = selected.iter().position(|it| it == &entity);
            match (pos, state) {
                (None, true) => selected.push(entity),
                (None, false) => {}
                (Some(_pos), true) => {}
                (Some(pos), false) => {
                    selected.remove(pos);
                }
            }

            *list = Arc::new(selected);
        });
    }

    pub fn toggle_entity_selection(&self, entity: AnyReadModel) {
        self.set_selected.update(|list| {
            let mut selected = list.as_ref().clone();

            let pos = selected.iter().position(|it| it == &entity);
            match pos {
                None => selected.push(entity),
                Some(pos) => {
                    selected.remove(pos);
                }
            }

            *list = Arc::new(selected);
        });
    }
}

#[component]
pub fn CrudListView(
    #[prop(into)] data_provider: Signal<CrudRestDataProvider>,
    #[prop(into)] headers: Signal<Vec<Header>>,
    #[prop(into)] order_by: Signal<IndexMap<AnyReadField, Order>>,
    #[prop(into)] field_renderer_registry: Signal<FieldRendererRegistry<AnyReadField>>,
    #[prop(into)] actions: Signal<Vec<CrudAction>>,
) -> impl IntoView {
    let instance_ctx = expect_context::<CrudInstanceContext>();

    let filter_open = RwSignal::new(false);
    let filter = RwSignal::new(Option::<String>::None);

    let read_allowed = Signal::derive(move || true);
    let edit_allowed = Signal::derive(move || true);
    let delete_allowed = Signal::derive(move || true);

    //let headers = Memo::new(move |_prev| {
    //    headers
    //        .get()
    //        .iter()
    //        .map(|Header { field, options }| (field.clone(), options.clone()))
    //        .collect::<Vec<(AnyField, HeaderOptions)>>() // ReadModel field
    //});

    let page_resource = LocalResource::new(move || async move {
        let _ = instance_ctx.reload.get();
        let items_per_page = instance_ctx.items_per_page.get().0;
        let page = instance_ctx.current_page.get().0;

        data_provider
            .get()
            .read_many(ReadMany {
                limit: Some(items_per_page),
                skip: Some(items_per_page * (page - 1)),
                order_by: Some({
                    let original = instance_ctx.order_by.get();
                    let mut new = IndexMap::new();
                    for (field, order) in original {
                        new.insert(SerializableReadField::from(field), order.clone());
                    }
                    new
                }),
                condition: instance_ctx.base_condition.get(),
            })
            .await
            .and_then(|json| {
                instance_ctx
                    .static_config
                    .read_value()
                    .model_handler
                    .deserialize_read_many_response
                    .run(json)
                    .map_err(|de_err| RequestError::Deserialize(de_err.to_string()))
            })
    });

    let page = Memo::new(move |_prev| match page_resource.get() {
        Some(result) => {
            tracing::trace!("loaded list data");
            match result {
                Ok(data) => Ok(Arc::new(data)),
                Err(reason) => Err(NoDataAvailable::RequestFailed(reason)),
            }
        }
        None => Err(NoDataAvailable::NotYetLoaded),
    });

    let count_resource = LocalResource::new(move || async move {
        let _ = instance_ctx.reload.get();
        data_provider
            .get()
            .read_count(ReadCount {
                condition: instance_ctx.base_condition.get(),
            })
            .await
    });

    let (selected, set_selected) = signal(Arc::new(Vec::<AnyReadModel>::new()));

    let list_view_context = CrudListViewContext {
        data: page,
        has_data: Signal::derive(move || {
            let data = page.read();
            data.is_ok() && data.as_ref().unwrap().len() > 0
        }),
        selected,
        set_selected,
        all_selected: Signal::derive(move || {
            let data = page.read();
            let selected = selected.get();
            data.is_ok() // TODO: Performance, memo?
                && selected.len() == data.as_ref().unwrap().len()
                && data.as_ref().unwrap().len() > 0
        }),
    };
    provide_context(list_view_context);

    // Clear the selection when the data is reloaded (e.g., after mass deletion).
    Effect::new(move || {
        let _ = instance_ctx.reload.get();
        list_view_context.clear_selection();
    });

    let multiselect_info = move || {
        selected.with(|selected| match selected.len() {
            0 => None,
            num_selected => {
                let selected_clone = selected.clone();
                Some(view! {
                    <div class="multiselect-actions">
                        <div>{num_selected} " ausgewählt"</div>
                        <Button
                            color=ButtonColor::Danger
                            on_press=move |_| {
                                instance_ctx.request_mass_deletion(selected_clone.clone())
                            }
                        >
                            <Icon icon=icondata::BsTrash/>
                            "Auswahl löschen"
                        </Button>
                    </div>
                })
            }
        })
    };

    view! {
        <ActionRow actions filter filter_open />

        <CrudTable
            headers=headers
            order_by=order_by
            data=page
            field_renderer_registry=field_renderer_registry
            read_allowed=read_allowed
            edit_allowed=edit_allowed
            delete_allowed=delete_allowed
            additional_item_actions=Signal::derive(move || vec![])
        />

        {multiselect_info}

        // Pagination
        {move || match count_resource.get() {
            Some(Ok(count)) => {
                view! {
                    <CrudPagination
                        item_count=count
                        items_per_page=instance_ctx.items_per_page
                        current_page=instance_ctx.current_page
                        set_current_page=move |page_number| instance_ctx.set_page(page_number)
                        set_items_per_page=move |item_count| instance_ctx.set_items_per_page(item_count)
                    />
                }.into_any()
            },
            Some(Err(reason)) => {
                view! { <div>{format!("Keine Daten verfügbar: {reason:?}")}</div> }.into_any()
            },
            None => view! {}.into_any(),
        }}
    }
}

#[component]
fn ActionRow(
    actions: Signal<Vec<CrudAction>>,
    filter: RwSignal<Option<String>>,
    filter_open: RwSignal<bool>,
) -> impl IntoView {
    let instance_ctx = expect_context::<CrudInstanceContext>();
    let action_ctx = CrudActionContext::new();
    view! {
        <Grid gap=Size::Em(0.6) attr:class="crud-nav">
            <Row>
                <Col xs=6>
                    <ButtonWrapper>
                        <Button color=ButtonColor::Success on_press=move |_| { instance_ctx.create() }>
                            <Icon icon=icondata::BsPlusCircle/>
                            <span style="text-decoration: underline">"N"</span>
                            "eu"
                        </Button>

                        <For
                            each=move || actions.get()
                            key=|action| action.id
                            children=move |CrudAction { id, name, icon, button_color, action, view }| {
                                if let Some(view_fn) = view {
                                    view! {
                                        <Button
                                            color=button_color
                                            disabled=Signal::derive(move || { action_ctx.is_action_executing(id) })
                                            on_press=move |_| action_ctx.request_action(id)
                                        >
                                            {icon.map(|icon| view! { <Icon icon=icon/> })}
                                            {name.clone()}
                                        </Button>
                                        {
                                            view_fn.run(ResourceActionViewInput {
                                                show_when: Signal::derive(move || {
                                                    action_ctx.is_action_requested(id)
                                                }),
                                                cancel: Callback::new(move |_| { action_ctx.cancel_action(id) }),
                                                execute: Callback::new(move |action_payload| {
                                                    action_ctx
                                                        .trigger_action(id, action_payload, action, instance_ctx)
                                                }),
                                            })
                                        }
                                    }.into_any()
                                } else {
                                    view! {
                                        <Button
                                            color=button_color
                                            disabled=Signal::derive(move || { action_ctx.is_action_executing(id) })
                                            on_press=move |_| {
                                                action_ctx.trigger_action(id, None, action, instance_ctx)
                                            }
                                        >
                                            {icon.map(|icon| view! { <Icon icon=icon/> })}
                                            {name.clone()}
                                        </Button>
                                    }.into_any()
                                }
                            }
                        />

                    </ButtonWrapper>
                </Col>
                <Col xs=6 h_align=ColAlign::End>
                    <ButtonWrapper>
                        <Button color=ButtonColor::Secondary on_press=move |_| { instance_ctx.reset() }>
                            <Icon icon=icondata::BsArrowRepeat/>
                            "Reset"
                        </Button>
                        <Button color=ButtonColor::Primary disabled=true on_press=move |_| filter_open.set(!filter_open.get_untracked())>
                            <Icon icon=icondata::BsSearch/>
                            "Filter"
                            {move || {
                                filter
                                    .get()
                                    .map(|_filter| {
                                        view! {
                                            <div style="font-size: 0.5em; font-weight: bold; margin-left: 0.3em;">
                                                "aktiv"
                                            </div>
                                        }
                                    })
                            }}
                        </Button>
                    </ButtonWrapper>
                </Col>
            </Row>
        </Grid>
    }
}
