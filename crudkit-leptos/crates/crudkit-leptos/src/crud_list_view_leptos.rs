use std::{marker::PhantomData, rc::Rc};

use crudkit_shared::Order;
use crudkit_web::prelude::*;
use indexmap::IndexMap;
use leptonic::prelude::*;
use leptos::*;
use leptos_icons::BsIcon;

use crate::{crud_instance_leptos::CrudInstanceContext, prelude::CrudTableL};

#[derive(Debug, Clone, PartialEq)]
struct PageReq<T: CrudMainTrait + 'static> {
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

#[derive(Debug, Clone)]
pub struct CrudListViewContext<T: CrudMainTrait + 'static> {
    pub data: Signal<Option<Rc<Vec<Rc<T::ReadModel>>>>>,
    pub has_data: Signal<bool>,
    pub selected: ReadSignal<Rc<Vec<Rc<T::ReadModel>>>>,
    set_selected: WriteSignal<Rc<Vec<Rc<T::ReadModel>>>>,
    pub all_selected: Signal<bool>,
}

impl<T: CrudMainTrait + 'static> CrudListViewContext<T> {
    pub fn toggle_select_all(&self) {
        if let Some(data) = self.data.get() {
            let selected = self.selected.get();
            if selected.len() < data.len() {
                // Select all
                self.set_selected.update(|selected| *selected = data.clone());
            } else {
                // Deselect all
                self.set_selected.update(|selected| *selected = Rc::new(Vec::new()));
            }
        } else {
            tracing::warn!("Tried to toggle_select_all when no data was present.");
        }
    }

    pub fn select(&self, entity: Rc<T::ReadModel>, state: bool) {
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

    pub fn toggle_entity_selection(&self, entity: Rc<T::ReadModel>) {
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
    #[prop(into)] items_per_page: Signal<u64>,
    #[prop(into)] page: Signal<u64>,
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
        tracing::info!(?order_by, "headers");
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

    // Whenever this signal changes, the currently viewed page should be re-fetched.
    let page_req = Signal::derive(cx, move || {
        let order_by = instance_ctx.order_by.get();
        let page = page.get();
        let items_per_page = items_per_page.get();
        let data_provider = data_provider.get();
        PageReq {
            order_by,
            page,
            items_per_page,
            data_provider,
        }
    });

    let page = create_local_resource(cx, page_req, load_page);

    // The data of the page when successfully loaded.
    let data = Signal::derive(cx, move || match page.read(cx) {
        Some(result) => match result {
            Ok(data) => Some(Rc::new(data.into_iter().map(Rc::new).collect())),
            Err(_err) => None,
        },
        None => None,
    });

    let (selected, set_selected) = create_signal(cx, Rc::new(Vec::<Rc<T::ReadModel>>::new()));

    provide_context(
        cx,
        CrudListViewContext::<T> {
            data,
            has_data: Signal::derive(cx, move || {
                let data = data.get();
                data.is_some() && data.as_ref().unwrap().len() > 0
            }),
            selected,
            set_selected,
            all_selected: Signal::derive(cx, move || {
                let data = data.get();
                let selected = selected.get();
                data.is_some() // TODO: Performance
                    && selected.len() == data.as_ref().unwrap().len()
                    && data.as_ref().unwrap().len() > 0
            }),
        },
    );

    let reset = move |e| {};
    let toggle_filter = move |e| {};
    let create = move |e| {};

    view! {cx,
        <Grid spacing=6 class="crud-nav">
            <Row>
                <Col xs=6>
                    <ButtonWrapper>
                        <Button color=ButtonColor::Success on_click=create>
                            <Icon icon=BsIcon::BsPlusCircle/>
                            <span style="text-decoration: underline">"N"</span> "eu"
                        </Button>

                        // TODO: implement from yew: ctx.props().static_config.actions.iter()
                    </ButtonWrapper>
                </Col>
                <Col xs=6 h_align=ColAlign::End>
                    <ButtonWrapper>
                        <Button color=ButtonColor::Secondary on_click=reset>
                            <Icon icon=BsIcon::BsArrowRepeat/>
                            "Reset"
                        </Button>
                        <Button color=ButtonColor::Primary disabled=true on_click=toggle_filter>
                            <Icon icon=BsIcon::BsSearch/>
                            "Filter"
                            { move || match filter.get() {
                                Some(_filter) => view! {cx,
                                    <div style="font-size: 0.5em; font-weight: bold; margin-left: 0.3em;">
                                        "aktiv"
                                    </div>
                                }.into_view(cx),
                                None => ().into_view(cx)
                            }}
                        </Button>
                    </ButtonWrapper>
                </Col>
            </Row>
        </Grid>

        // TODO: CrudTable
        <CrudTableL
            _phantom={PhantomData::<T>::default()}
            api_base_url=api_base_url
            headers=headers
            custom_fields=custom_fields
            //data=data
            read_allowed=read_allowed
            edit_allowed=edit_allowed
            delete_allowed=delete_allowed
            additional_item_actions=Signal::derive(cx, move || vec![])
        />

        // TODO: Multiselect actions

        // TODO: Pagination
    }
}
