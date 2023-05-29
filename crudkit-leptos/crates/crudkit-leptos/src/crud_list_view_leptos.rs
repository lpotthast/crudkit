use std::{marker::PhantomData, rc::Rc};

use crudkit_shared::Order;
use crudkit_web::prelude::*;
use indexmap::IndexMap;
use leptonic::prelude::*;
use leptos::*;
use leptos_icons::BsIcon;
use uuid::Uuid;

use crate::{
    crud_action::{Callback, CrudAction, CrudActionAftermath, ModalGeneration},
    crud_instance_leptos::CrudInstanceContext,
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

async fn load_page<T: CrudMainTrait + 'static>(
    req: PageReq<T>,
) -> Result<Vec<T::ReadModel>, RequestError> {
    req.data_provider
        .read_many(ReadMany {
            limit: Some(req.items_per_page),
            skip: Some(req.items_per_page * (req.page - 1)),
            order_by: Some(req.order_by),
            condition: None,
        })
        .await
}

#[derive(Debug, Clone, PartialEq)]
struct CountReq<T: CrudMainTrait + 'static> {
    reload: Uuid,
    data_provider: CrudRestDataProvider<T>,
}

async fn load_count<T: CrudMainTrait + 'static>(req: CountReq<T>) -> Result<u64, RequestError> {
    req.data_provider
        .read_count(ReadCount { condition: None })
        .await
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
pub fn CrudListViewL<T>(
    cx: Scope,
    data_provider: Signal<CrudRestDataProvider<T>>,
    #[prop(into)] api_base_url: Signal<String>,
    #[prop(into)] headers: Signal<Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions)>>,
    #[prop(into)] custom_fields: Signal<CustomReadFields<T, leptos::View>>,
    #[prop(into)] actions: Signal<Vec<CrudAction<T>>>,
    //config: Signal<CrudInstanceConfig<T>>,
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
        let order_by = instance_ctx.order_by.get();
        //tracing::debug!(?order_by, "headers");
        headers
            .get()
            .iter()
            .map(|(field, options)| (field.clone(), options.clone(), order_by.get(field).cloned()))
            .collect::<Vec<(
                <T::ReadModel as CrudDataTrait>::Field,
                HeaderOptions,
                Option<Order>,
            )>>()
    });

    // Whenever this signal returns a new/different value, the currently viewed page is re-fetched.
    let page_req = Signal::derive(cx, move || {
        tracing::debug!("page_req");
        // Every server-provided resource must be reloaded when a general reload is requested!
        let reload = instance_ctx.reload.get();
        let order_by = instance_ctx.order_by.get();
        let page = instance_ctx.current_page.get();
        let items_per_page = instance_ctx.items_per_page.get();
        let data_provider = data_provider.get();
        PageReq {
            reload,
            order_by,
            page,
            items_per_page,
            data_provider,
        }
    });

    // Whenever this signal returns a new/different value, the item-count is re-fetched.
    let count_req = Signal::derive(cx, move || {
        tracing::debug!("count_req");
        // Every server-provided resource must be reloaded when a general reload is requested!
        let reload = instance_ctx.reload.get();
        let data_provider = data_provider.get();
        CountReq { reload, data_provider }
    });

    let page_res = create_local_resource(cx, move || page_req.get(), load_page);
    let count_res = create_local_resource(cx, move || count_req.get(), load_count);

    // The data of the page when successfully loaded.
    // TODO: create_memo or Signal::derive??? We only want this once..
    let data = create_memo(cx, move |_prev| match page_res.read(cx) {
        Some(result) => {
            tracing::info!("loaded list data");
            match result {
                Ok(data) => Ok(Rc::new(data)),
                Err(reason) => Err(NoDataAvailable::FetchFailed(reason)),
            }
        }
        None => Err(NoDataAvailable::NotYetLoaded),
    });

    let count = Signal::derive(cx, move || count_res.read(cx));

    let (selected, set_selected) = create_signal(cx, Rc::new(Vec::<T::ReadModel>::new()));

    provide_context(
        cx,
        CrudListViewContext::<T> {
            data,
            has_data: Signal::derive(cx, move || {
                let data = data.get();
                data.is_ok() && data.as_ref().unwrap().len() > 0
            }),
            selected,
            set_selected,
            all_selected: Signal::derive(cx, move || {
                let data = data.get();
                let selected = selected.get();
                data.is_ok() // TODO: Performance, memo?
                    && selected.len() == data.as_ref().unwrap().len()
                    && data.as_ref().unwrap().len() > 0
            }),
        },
    );

    let toggle_filter = move |e| {};

    // Stores the actions for which execution was requested by the user.
    let (actions_requested, set_actions_requested) = create_signal(cx, Vec::<String>::new());

    let is_action_requested = move |action_id| {
        Signal::derive(cx, move || {
            actions_requested
                .get()
                .iter()
                .any(|it| it.as_str() == action_id)
        })
    };

    // Stores the actions which are currently executing. This allows us to not let a user execute an action while it is already executing.
    let (actions_executing, set_actions_executing) = create_signal(cx, Vec::new());

    let is_action_executing = move |action_id: String| actions_executing.get().contains(&action_id);

    let request_action = move |action_id| {
        tracing::info!(action_id, "request_action");
        set_actions_requested.update(|actions| actions.push(action_id));
    };

    let cancel_action = move |action_id| {
        tracing::info!(action_id, "cancel_action");
        set_actions_requested.update(|actions| {
            let pos = actions.iter().position(|id| id == action_id);
            if let Some(pos) = pos {
                actions.remove(pos);
            }
        });
    };

    let trigger_action = move |action_id: &'static str,
                               action_payload,
                               action: Callback<(
        Option<T::ActionPayload>,
        Callback<Result<CrudActionAftermath, CrudActionAftermath>>,
    )>| {
        tracing::info!(action_id, ?action_payload, "trigger_action");

        // The user accepted the request. The action is no longer requested.
        set_actions_requested.update(|actions| {
            let pos = actions.iter().position(|id| id == action_id);
            if let Some(pos) = pos {
                actions.remove(pos);
            }
        });

        // We are going to execute the action and track that here.
        set_actions_executing.update(|actions| actions.push(action_id.to_owned()));

        action.0((
            action_payload,
            Callback::of(move |outcome| {
                tracing::info!(?outcome, "action finished");

                // Regardless of the outcome, the action is now finished.
                set_actions_executing.update(|actions| {
                    let pos = actions.iter().position(|id| id == action_id);
                    if let Some(pos) = pos {
                        actions.remove(pos);
                    }
                });

                // We might have to act in a way that this list view can not comprehend and therefore let the instance handle the outcome.
                expect_context::<CrudInstanceContext<T>>(cx).handle_action_outcome(cx, outcome);
            }),
        ));
    };

    let action_row = view! {cx,
        <Grid spacing=6 class="crud-nav">
            <Row>
                <Col xs=6>
                    <ButtonWrapper>
                        <Button color=ButtonColor::Success on_click=move |_| { expect_context::<CrudInstanceContext<T>>(cx).create() }>
                            <Icon icon=BsIcon::BsPlusCircle/>
                            <span style="text-decoration: underline">"N"</span> "eu"
                        </Button>

                        { move || actions.get().into_iter()
                                .map(|action| match action {
                                    CrudAction::Custom {id, name, icon, button_color, action, modal} => {
                                        let action_id: &'static str = id; // TODO: remove
                                        let action = action.clone();

                                        if let Some(modal) = modal {
                                            view! {cx,
                                                <Button
                                                    color=button_color
                                                    disabled=is_action_executing(action_id.to_owned())
                                                    on_click=move |_| request_action(action_id.to_owned())
                                                >
                                                    { icon.map(|icon| view! {cx, <Icon icon=icon/>}) }
                                                    { name.clone() }
                                                </Button>
                                                {
                                                    modal.0((cx, ModalGeneration {
                                                        show_when: is_action_requested(action_id),
                                                        cancel: Callback::of(move |_| cancel_action(action_id.clone())),
                                                        execute: Callback::of(move |action_payload| trigger_action(action_id, action_payload, action.clone())),
                                                    }))
                                                }
                                            }.into_view(cx)
                                        } else {
                                            view! {cx,
                                                <Button
                                                    color=button_color
                                                    disabled=is_action_executing(action_id.to_owned())
                                                    on_click=move |_| trigger_action(action_id, None, action.clone())
                                                >
                                                    { icon.map(|icon| view! {cx, <Icon icon=icon/>}) }
                                                    { name.clone() }
                                                </Button>
                                            }.into_view(cx)
                                        }
                                    }
                                }).collect_view(cx)
                        }
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

    let pagination = move || {
        match count.get() {
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
    }
    };

    view! {cx,
        { action_row }

        <CrudTable
            _phantom={PhantomData::<T>::default()}
            api_base_url=api_base_url
            headers=headers
            data=data
            custom_fields=custom_fields
            read_allowed=read_allowed
            edit_allowed=edit_allowed
            delete_allowed=delete_allowed
            additional_item_actions=Signal::derive(cx, move || vec![])
        />

        { multiselect_info }

        { pagination }
    }
}
