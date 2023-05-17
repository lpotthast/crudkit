use std::{marker::PhantomData, rc::Rc};

use crudkit_shared::Order;
use crudkit_web::prelude::*;
use indexmap::IndexMap;
use leptonic::prelude::*;
use leptos::*;
use leptos_icons::BsIcon;

use crate::prelude::CrudTableL;

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

#[component]
pub fn CrudListViewL<T>(
    cx: Scope,
    data_provider: Signal<CrudRestDataProvider<T>>,
    #[prop(into)] headers: Signal<Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions)>>,
    #[prop(into)] order_by: Signal<IndexMap<<T::ReadModel as CrudDataTrait>::Field, Order>>,
    #[prop(into)] items_per_page: Signal<u64>,
    #[prop(into)] page: Signal<u64>,
    //config: Signal<CrudInstanceConfig<T>>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let (filter, set_filter) = create_signal(cx, Option::<String>::None);

    let read_allowed = Signal::derive(cx, move || true);
    let edit_allowed = Signal::derive(cx, move || true);
    let delete_allowed = Signal::derive(cx, move || true);

    let headers = Signal::derive(cx, move || {
        let order_by = order_by.get();
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
        let order_by = order_by.get();
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
    let data = Signal::derive(cx, move || {
        let p = page.read(cx);
        match p {
            Some(result) => match result {
                Ok(data) => Some(Rc::new(data)),
                Err(_err) => None,
            },
            None => None,
        }
    });

    let (selected, set_selected) = create_signal(cx, Vec::<T::ReadModel>::new());

    let reset = move |e| {};
    let toggle_filter = move |e| {};
    let create = move |e| {};

    let phantom = PhantomData::<T::ReadModel>::default();

    view! {cx,
        "ListView"
        <Grid spacing=6>
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
            phantom=phantom
            headers=headers
            data=data
            selected=selected
            read_allowed=read_allowed
            edit_allowed=edit_allowed
            delete_allowed=delete_allowed
            additional_item_actions=Signal::derive(cx, move || vec![])
        />

        // TODO: Multiselect actions

        // TODO: Pagination
    }
}
