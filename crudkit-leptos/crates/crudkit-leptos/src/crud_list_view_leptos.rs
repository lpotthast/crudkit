use std::{rc::Rc, marker::PhantomData};

use crudkit_shared::Order;
use crudkit_web::prelude::*;
use indexmap::IndexMap;
use leptonic::prelude::*;
use leptos::*;
use leptos_icons::BsIcon;

use crate::prelude::CrudTableL;

#[component]
pub fn CrudListViewL<T>(
    cx: Scope,
    data_provider: Signal<CrudRestDataProvider<T>>,
    #[prop(into)] headers: Signal<Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions)>>,
    #[prop(into)] order_by: Signal<IndexMap<<T::ReadModel as CrudDataTrait>::Field, Order>>,
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
        headers.get().iter()
            .map(|(field, options)| (field.clone(), options.clone(), order_by.get(field).cloned()))
            .collect::<Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions, Option<Order>)>>()
    });

    let (data, set_data) = create_signal(cx, Option::<Rc<Vec<T::ReadModel>>>::None);

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
