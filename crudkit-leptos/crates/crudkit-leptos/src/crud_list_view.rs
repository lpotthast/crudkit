use std::{marker::PhantomData, rc::Rc};

use crudkit_shared::Order;
use crudkit_web::prelude::*;
use indexmap::IndexMap;
use leptonic::prelude::*;
use leptos::*;
use leptos_icons::BsIcon;
use uuid::Uuid;

use crate::{
    crud_action::{Callable, Callback, CrudAction, ModalGeneration},
    crud_action_context::CrudActionContext,
    crud_instance::CrudInstanceContext,
    crud_pagination::CrudPagination,
    crud_table::NoDataAvailable,
    prelude::CrudTable,
};

#[derive(Debug, Clone, PartialEq)]
struct PageReq<T: CrudMainTrait + 'static> {
    reload: Uuid,
    order_by: IndexMap<<T::ReadModel as CrudDataTrait>::Field, Order>,
    page: u64,
    items_per_page: u64,
    data_provider: CrudRestDataProvider<T>,
}

#[derive(Debug, Clone, PartialEq)]
struct CountReq<T: CrudMainTrait + 'static> {
    reload: Uuid,
    data_provider: CrudRestDataProvider<T>,
}

#[derive(Debug, Clone)]
pub struct CrudListViewContext<T: CrudMainTrait + 'static> {
    pub data: Memo<Result<Rc<Vec<T::ReadModel>>, NoDataAvailable>>,
    pub has_data: Signal<bool>,

    pub selected: ReadSignal<Rc<Vec<T::ReadModel>>>,
    set_selected: WriteSignal<Rc<Vec<T::ReadModel>>>,
    pub all_selected: Signal<bool>,
}

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
                    .update(|selected| *selected = Rc::new(Vec::new()));
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

            *list = Rc::new(selected);
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

            *list = Rc::new(selected);
        });
    }
}

#[component]
pub fn CrudListView<T>(
    cx: Scope,
    #[prop(into)] data_provider: Signal<CrudRestDataProvider<T>>,
    #[prop(into)] api_base_url: Signal<String>,
    #[prop(into)] headers: Signal<Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions)>>,
    #[prop(into)] order_by: Signal<IndexMap<<T::ReadModel as CrudDataTrait>::Field, Order>>,
    #[prop(into)] custom_fields: Signal<CustomReadFields<T, leptos::View>>,
    #[prop(into)] actions: Signal<Vec<CrudAction<T>>>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let instance_ctx = expect_context::<CrudInstanceContext<T>>(cx);

    let (filter, set_filter) = create_signal(cx, Option::<String>::None);

    let read_allowed = Signal::derive(cx, move || true);
    let edit_allowed = Signal::derive(cx, move || true);
    let delete_allowed = Signal::derive(cx, move || true);

    let headers = create_memo(cx, move |_prev| {
        tracing::debug!("headers");
        headers
            .get()
            .iter()
            .map(|(field, options)| (field.clone(), options.clone()))
            .collect::<Vec<(
                <T::ReadModel as CrudDataTrait>::Field,
                HeaderOptions,
            )>>()
    });

    let page_resource = create_local_resource(
        cx,
        move || {
            tracing::debug!("page_req");
            PageReq {
                reload: instance_ctx.reload.get(),
                order_by: instance_ctx.order_by.get(),
                page: instance_ctx.current_page.get(),
                items_per_page: instance_ctx.items_per_page.get(),
                data_provider: data_provider.get(),
            }
        },
        move |req| async move {
            req.data_provider
                .read_many(ReadMany {
                    limit: Some(req.items_per_page),
                    skip: Some(req.items_per_page * (req.page - 1)),
                    order_by: Some(req.order_by),
                    condition: None,
                })
                .await
        },
    );

    let page = create_memo(cx, move |_prev| match page_resource.read(cx) {
        Some(result) => {
            tracing::info!("loaded list data");
            match result {
                Ok(data) => Ok(Rc::new(data)),
                Err(reason) => Err(NoDataAvailable::RequestFailed(reason)),
            }
        }
        None => Err(NoDataAvailable::NotYetLoaded),
    });

    let count_resource = create_local_resource(
        cx,
        move || {
            tracing::debug!("count_req");
            CountReq {
                reload: instance_ctx.reload.get(),
                data_provider: data_provider.get(),
            }
        },
        move |req| async move {
            req.data_provider
                .read_count(ReadCount { condition: None })
                .await
        },
    );

    let count = Signal::derive(cx, move || count_resource.read(cx));

    let (selected, set_selected) = create_signal(cx, Rc::new(Vec::<T::ReadModel>::new()));

    provide_context(
        cx,
        CrudListViewContext::<T> {
            data: page,
            has_data: Signal::derive(cx, move || {
                let data = page.get();
                data.is_ok() && data.as_ref().unwrap().len() > 0
            }),
            selected,
            set_selected,
            all_selected: Signal::derive(cx, move || {
                let data = page.get();
                let selected = selected.get();
                data.is_ok() // TODO: Performance, memo?
                    && selected.len() == data.as_ref().unwrap().len()
                    && data.as_ref().unwrap().len() > 0
            }),
        },
    );

    let toggle_filter = move |e| {};

    let action_ctx = CrudActionContext::<T>::new(cx);

    let action_row = view! {cx,
        <Grid spacing=6 class="crud-nav">
            <Row>
                <Col xs=6>
                    <ButtonWrapper>
                        <Button color=ButtonColor::Success on_click=move |_| { expect_context::<CrudInstanceContext<T>>(cx).create() }>
                            <Icon icon=BsIcon::BsPlusCircle/>
                            <span style="text-decoration: underline">"N"</span> "eu"
                        </Button>

                        <For
                            each=move || actions.get()
                            key=|action| match action {
                                CrudAction::Custom {id, name: _, icon: _, button_color: _, action: _, modal: _} => *id,
                            }
                            view=move |cx, action| match action {
                                CrudAction::Custom {id, name, icon, button_color, action, modal} => {
                                    if let Some(modal_generator) = modal {
                                        view! {cx,
                                            <Button
                                                color=button_color
                                                disabled=Signal::derive(cx, move || action_ctx.is_action_executing(id))
                                                on_click=move |_| action_ctx.request_action(id)
                                            >
                                                { icon.map(|icon| view! {cx, <Icon icon=icon/>}) }
                                                { name.clone() }
                                            </Button>
                                            {
                                                modal_generator.call_with((cx, ModalGeneration {
                                                    show_when: Signal::derive(cx, move || action_ctx.is_action_requested(id)),
                                                    cancel: Callback::new(cx, move |_| action_ctx.cancel_action(id)),
                                                    execute: Callback::new(cx, move |action_payload| action_ctx.trigger_action(cx, id, action_payload, action)),
                                                }))
                                            }
                                        }.into_view(cx)
                                    } else {
                                        view! {cx,
                                            <Button
                                                color=button_color
                                                disabled=Signal::derive(cx, move || action_ctx.is_action_executing(id))
                                                on_click=move |_| action_ctx.trigger_action(cx, id, None, action)
                                            >
                                                { icon.map(|icon| view! {cx, <Icon icon=icon/>}) }
                                                { name.clone() }
                                            </Button>
                                        }.into_view(cx)
                                    }
                                }
                            }
                        />
                    </ButtonWrapper>
                </Col>
                <Col xs=6 h_align=ColAlign::End>
                    <ButtonWrapper>
                        <Button color=ButtonColor::Secondary on_click=move |_| { expect_context::<CrudInstanceContext<T>>(cx).reset() }>
                            <Icon icon=BsIcon::BsArrowRepeat/>
                            "Reset"
                        </Button>
                        <Button color=ButtonColor::Primary disabled=true on_click=toggle_filter>
                            <Icon icon=BsIcon::BsSearch/>
                            "Filter"
                            { move || filter.get().map(|_filter| view! {cx,
                                <div style="font-size: 0.5em; font-weight: bold; margin-left: 0.3em;">
                                    "aktiv"
                                </div>
                            }) }
                        </Button>
                    </ButtonWrapper>
                </Col>
            </Row>
        </Grid>
    };

    let multiselect_info = move || match selected.get().len() {
        0 => None,
        num_selected => Some(view! {cx,
            <div class="multiselect-actions">
                <div>
                    { num_selected } " selected"
                </div>
            </div>
        }),
    };

    view! {cx,
        { action_row }

        <CrudTable
            _phantom={PhantomData::<T>::default()}
            api_base_url=api_base_url
            headers=headers
            order_by=order_by
            data=page
            custom_fields=custom_fields
            read_allowed=read_allowed
            edit_allowed=edit_allowed
            delete_allowed=delete_allowed
            additional_item_actions=Signal::derive(cx, move || vec![])
        />

        { multiselect_info }

        // Pagination
        { move || match count.get() {
            Some(result) => match result {
                Ok(count) => Some(
                    view! {cx,
                        <CrudPagination
                            current_page=instance_ctx.current_page
                            item_count=count
                            items_per_page=instance_ctx.items_per_page
                            on_page_select=move |page_number| {
                                expect_context::<CrudInstanceContext<T>>(cx).set_page(page_number)
                            }
                            on_item_count_select=move |item_count| {
                                expect_context::<CrudInstanceContext<T>>(cx).set_items_per_page(item_count)
                            }
                        />
                    }
                    .into_view(cx),
                ),
                Err(reason) => Some(
                    view! {cx,
                        <div>
                            {format!("Keine Daten verfügbar: {reason:?}") }
                        </div>
                    }
                    .into_view(cx),
                ),
            },
            None => None,
        } }
    }
}
