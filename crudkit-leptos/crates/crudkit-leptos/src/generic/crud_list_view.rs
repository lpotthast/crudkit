use crate::generic::crud_action::{CrudAction, ModalGeneration};
use crate::generic::crud_action_context::CrudActionContext;
use crate::generic::crud_instance::CrudInstanceContext;
use crate::generic::crud_instance_config::DynSelectConfig;
use crate::generic::crud_table::{CrudTable, NoDataAvailable};
use crate::generic::custom_field::CustomReadFields;
use crate::shared::crud_pagination::CrudPagination;
use crudkit_shared::Order;
use crudkit_web::prelude::*;
use indexmap::IndexMap;
use leptonic::components::prelude::*;
use leptonic::prelude::*;
use leptos::prelude::*;
use std::sync::Arc;
use std::{collections::HashMap, marker::PhantomData};

#[derive(Debug, Clone)]
pub struct CrudListViewContext<T: CrudMainTrait + 'static> {
    pub data: Memo<Result<Arc<Vec<T::ReadModel>>, NoDataAvailable>>,
    pub has_data: Signal<bool>,

    pub selected: ReadSignal<Arc<Vec<T::ReadModel>>>,
    set_selected: WriteSignal<Arc<Vec<T::ReadModel>>>,
    pub all_selected: Signal<bool>,
}

impl<T: CrudMainTrait + 'static> Copy for CrudListViewContext<T> {}

impl<T: CrudMainTrait + 'static> CrudListViewContext<T> {
    pub fn toggle_select_all(&self) {
        if let Ok(data) = self.data.get() {
            let selected = self.selected.get();
            if selected.len() < data.len() {
                // Select all
                self.set_selected
                    .update(|selected| *selected = data.clone());
            } else {
                // Deselect all
                self.set_selected
                    .update(|selected| *selected = Arc::new(Vec::new()));
            }
        } else {
            tracing::warn!("Tried to toggle_select_all when no data was present.");
        }
    }

    pub fn select(&self, entity: T::ReadModel, state: bool) {
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

    pub fn toggle_entity_selection(&self, entity: T::ReadModel) {
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
pub fn CrudListView<T>(
    #[prop(into)] data_provider: Signal<CrudRestDataProvider<T>>,
    #[prop(into)] api_base_url: Signal<String>,
    #[prop(into)] headers: Signal<Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions)>>,
    #[prop(into)] order_by: Signal<IndexMap<<T::ReadModel as CrudDataTrait>::Field, Order>>,
    #[prop(into)] custom_fields: Signal<CustomReadFields<T>>,
    #[prop(into)] field_config: Signal<
        HashMap<<T::ReadModel as CrudDataTrait>::Field, DynSelectConfig>,
    >,
    #[prop(into)] actions: Signal<Vec<CrudAction<T>>>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let instance_ctx = expect_context::<CrudInstanceContext<T>>();

    let filter_open = RwSignal::new(false);
    let filter = RwSignal::new(Option::<String>::None);

    let read_allowed = Signal::derive(move || true);
    let edit_allowed = Signal::derive(move || true);
    let delete_allowed = Signal::derive(move || true);

    let headers = Memo::new(move |_prev| {
        headers
            .get()
            .iter()
            .map(|(field, options)| (field.clone(), options.clone()))
            .collect::<Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions)>>()
    });

    // TODO: Investigate: After a fresh page load, the first change to one of the signals (items_per_page, ...) triggers an unexpected reload. source is not re-run.
    let page_resource = LocalResource::new(move || async move {
        let _ = instance_ctx.reload.get();

        let items_per_page = instance_ctx.items_per_page.get();
        let page = instance_ctx.current_page.get();

        data_provider
            .get()
            .read_many(ReadMany {
                limit: Some(items_per_page),
                skip: Some(items_per_page * (page - 1)),
                order_by: Some(instance_ctx.order_by.get()),
                condition: instance_ctx.base_condition.get(),
            })
            .await
    });

    let page = Memo::new(move |_prev| match page_resource.get() {
        Some(result) => {
            tracing::debug!("loaded list data");
            match result.take() {
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

    let (selected, set_selected) = signal(Arc::new(Vec::<T::ReadModel>::new()));

    provide_context(CrudListViewContext::<T> {
        data: page,
        has_data: Signal::derive(move || {
            let data = page.get();
            data.is_ok() && data.as_ref().unwrap().len() > 0
        }),
        selected,
        set_selected,
        all_selected: Signal::derive(move || {
            let data = page.get();
            let selected = selected.get();
            data.is_ok() // TODO: Performance, memo?
                    && selected.len() == data.as_ref().unwrap().len()
                    && data.as_ref().unwrap().len() > 0
        }),
    });

    let multiselect_info = move || {
        selected.with(|selected| match selected.len() {
            0 => None,
            num_selected => Some(view! {
                <div class="multiselect-actions">
                    <div>{num_selected} " selected"</div>
                </div>
            }),
        })
    };

    view! {
        <ActionRow actions filter filter_open />

        <CrudTable
            _phantom={PhantomData::<T>::default()}
            api_base_url=api_base_url
            headers=headers
            order_by=order_by
            data=page
            custom_fields=custom_fields
            field_config=field_config
            read_allowed=read_allowed
            edit_allowed=edit_allowed
            delete_allowed=delete_allowed
            additional_item_actions=Signal::derive(move || vec![])
        />

        {multiselect_info}

        // Pagination
        {move || match count_resource.get().deref() {
            Some(Ok(count)) => {
                view! {
                    <CrudPagination
                        item_count=*count
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
fn ActionRow<T>(
    actions: Signal<Vec<CrudAction<T>>>,
    filter: RwSignal<Option<String>>,
    filter_open: RwSignal<bool>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let instance_ctx = expect_context::<CrudInstanceContext<T>>();
    let action_ctx = CrudActionContext::<T>::new();
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
                            key=|action| match action {
                                CrudAction::Custom { id, name: _, icon: _, button_color: _, action: _, modal: _ } => *id,
                            }
                            children=move |action| match action {
                                CrudAction::Custom { id, name, icon, button_color, action, modal } => {
                                    if let Some(modal_generator) = modal {
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
                                                modal_generator.run(ModalGeneration {
                                                    show_when: Signal::derive(move || {
                                                        action_ctx.is_action_requested(id)
                                                    }),
                                                    cancel: Callback::new(move |_| { action_ctx.cancel_action(id) }),
                                                    execute: Callback::new(move |action_payload| {
                                                        action_ctx
                                                            .trigger_action(id, action_payload, action.clone(), instance_ctx)
                                                    }),
                                                })
                                            }
                                        }.into_any()
                                    } else {
                                        let action = action.clone();
                                        view! {
                                            <Button
                                                color=button_color
                                                disabled=Signal::derive(move || { action_ctx.is_action_executing(id) })
                                                on_press=move |_| {
                                                    action_ctx.trigger_action(id, None, action.clone(), instance_ctx)
                                                }
                                            >
                                                {icon.map(|icon| view! { <Icon icon=icon/> })}
                                                {name.clone()}
                                            </Button>
                                        }.into_any()
                                    }
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
