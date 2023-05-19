use std::{marker::PhantomData, rc::Rc};

use crudkit_shared::Order;
use crudkit_web::prelude::*;
use leptonic::prelude::*;
use leptos::*;
use uuid::Uuid;

// TODO: Add prelude entry for CrudActionTrait
use crate::{crud_action::CrudActionTrait, prelude::CrudTableHeaderL};

// TODO: Analyze what data is copied when.
#[component]
pub fn CrudTableL<T>(
    cx: Scope,
    phantom: PhantomData<T>,
    #[prop(into)] headers: Signal<Vec<(T::Field, HeaderOptions, Option<Order>)>>,
    #[prop(into)] data: Signal<Option<Rc<Vec<T>>>>,
    #[prop(into)] selected: Signal<Vec<T>>, // TODO: Does this also need to be Rc?
    #[prop(into)] read_allowed: Signal<bool>,
    #[prop(into)] edit_allowed: Signal<bool>,
    #[prop(into)] delete_allowed: Signal<bool>,
    #[prop(into)] additional_item_actions: Signal<Vec<Rc<Box<dyn CrudActionTrait>>>>,
) -> impl IntoView
where
    T: CrudDataTrait + 'static,
{
    let with_actions = Signal::derive(cx, move || {
        !additional_item_actions.get().is_empty()
            || read_allowed.get()
            || edit_allowed.get()
            || delete_allowed.get()
    });

    let has_data = Signal::derive(cx, move || {
        let data = data.get();
        data.is_some() && data.as_ref().unwrap().len() > 0
    });

    let all_selected = Signal::derive(cx, move || {
        let data = data.get();
        let selected = selected.get();
        data.is_some() // TODO: Performance
            && selected.len() == data.as_ref().unwrap().len()
            && data.as_ref().unwrap().len() > 0
    });

    let on_edit = move |entity: T| {};

    let select_entity = move |entity: T| {};

    let entity = move |cx, entity: T| {
        view! {cx,
            <tr class="interactable" on:click=move |_e| on_edit(entity.clone())>
                <td class="select" on:click=move |e| { e.stop_propagation() }>
                    <Checkbox
                        checked=Signal::derive(cx, move || selected.get().iter().find(|it| *it == &entity).is_some())
                        on_toggle=move || select_entity(entity) // TODO: also pass current state
                    />
                </td>
            </tr>
        }
    };

    let body = view! {cx,
        <tbody>
            {move || match data.get() {
                Some(data) => match data.len() {
                    0 => view! {cx,
                        <tr>
                            <td colspan={"100%"} class={"no-data"}>
                                {"Keine Daten"}
                            </td>
                        </tr>
                    }.into_view(cx),
                    _ => view! {cx,
                        <For
                            each=move || data.as_ref().clone() // TODO: Performance? Remove Rc in type?
                            key=|_entity| Uuid::new_v4() // TODO: Use entitiy!
                            view=entity
                        />
                    }.into_view(cx),
                },
                None => view! {cx,
                    <tr>
                        <td colspan="100%">
                            {"\u{00a0}"} // nbsp, see https://doc.rust-lang.org/std/primitive.char.html
                        </td>
                    </tr>
                }.into_view(cx),
            }}
        </tbody>
    };

    // TODO: Extract to leptonic
    view! {cx,
        "Table"
        <div class={"crud-table-wrapper"}>
            <table class={"crud-table crud-table-bordered crud-table-hoverable"}>
                // Header
                <CrudTableHeaderL
                    phantom=phantom
                    headers=headers
                    with_actions=with_actions
                    with_select_column=has_data
                    all_selected=all_selected
                />

                // Body
                {body}

                // Footer
            </table>
        </div>
    }
}
