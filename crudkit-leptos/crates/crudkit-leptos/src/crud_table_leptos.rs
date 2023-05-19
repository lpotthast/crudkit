use std::{marker::PhantomData, rc::Rc};

use crudkit_shared::Order;
use crudkit_web::prelude::*;
use leptonic::prelude::*;
use leptos::*;
use leptos_icons::BsIcon;
use uuid::Uuid;

// TODO: Add prelude entry for CrudActionTrait
use crate::{
    crud_action::CrudActionTrait,
    crud_field_leptos::CrudFieldL,
    crud_instance_leptos::CrudInstanceContext,
    crud_list_view_leptos::CrudListViewContext,
    prelude::{CrudTableFooterL, CrudTableHeaderL},
};

// TODO: Analyze what data is copied when.
#[component]
pub fn CrudTableL<T>(
    cx: Scope,
    _phantom: PhantomData<T>,
    #[prop(into)] api_base_url: Signal<String>,
    #[prop(into)] headers: Signal<
        Vec<(
            <T::ReadModel as CrudDataTrait>::Field,
            HeaderOptions,
            Option<Order>,
        )>,
    >,
    #[prop(into)] custom_fields: Signal<CustomFields<T::ReadModel, leptos::View>>,
    #[prop(into)] read_allowed: Signal<bool>,
    #[prop(into)] edit_allowed: Signal<bool>,
    #[prop(into)] delete_allowed: Signal<bool>,
    #[prop(into)] additional_item_actions: Signal<Vec<Rc<Box<dyn CrudActionTrait>>>>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let instance_ctx: CrudInstanceContext<T> = expect_context::<CrudInstanceContext<T>>(cx);
    let list_ctx: CrudListViewContext<T> = expect_context::<CrudListViewContext<T>>(cx);

    let with_actions = Signal::derive(cx, move || {
        !additional_item_actions.get().is_empty()
            || read_allowed.get()
            || edit_allowed.get()
            || delete_allowed.get()
    });

    let render_entry = move |cx, entity: Rc<T::ReadModel>| {
        // TODO: Check https://github.com/rust-lang/rfcs/issues/2407, we might be able to remove explicit clones in the future!
        let entity_clone1 = entity.clone();
        let entity_clone2 = entity.clone();
        let entity_clone3: Rc<<T as CrudMainTrait>::ReadModel> = entity.clone();
        let entity_clone4: Rc<<T as CrudMainTrait>::ReadModel> = entity.clone();
        let entity_clone5: Rc<<T as CrudMainTrait>::ReadModel> = entity.clone();

        let is_selected = create_memo(cx, move |_prev| {
            list_ctx
                .selected
                .get()
                .iter()
                .find(|it| *it == &entity_clone2)
                .is_some()
        });
        let toggle_selected = move || {
            expect_context::<CrudListViewContext<T>>(cx)
                .toggle_entity_selection(entity_clone3.clone())
        };

        view! {cx,
            <tr class="interactable"
                on:click=move |_e| { expect_context::<CrudInstanceContext<T>>(cx).edit(entity_clone1.as_ref().clone().into()) }
            >
                <td class="select" on:click=move |e| e.stop_propagation()>
                    <Checkbox
                        checked=is_selected
                        on_toggle=toggle_selected
                    />
                </td>

                <For
                    each=move || headers.get() // TODO: Performance? Remove Rc in type?
                    key=|(field, _, _)| field.get_name() // TODO: Is this unique enough?
                    view=move |cx, (field, options, _order)| view! {cx,
                        <td>
                            <CrudFieldL
                                //children={ctx.props().children.clone()} // TODO: make this work
                                custom_fields=custom_fields
                                api_base_url=api_base_url
                                current_view=CrudSimpleView::List
                                field=field.clone()
                                field_options=FieldOptions { disabled: false, label: None, date_time_display: options.date_time_display }
                                entity=entity.clone()
                                field_mode=FieldMode::Display
                                value_changed=|_, _| {}
                            />
                        </td>
                    }
                />

                { move || {
                    let entity_clone4 = entity_clone4.clone();
                    let entity_clone5 = entity_clone5.clone();
                    match with_actions.get() {
                        true => view! {cx,
                            <td on:click=move |e| { e.stop_propagation() }>
                                <div class="action-icons">
                                    {
                                        if read_allowed.get() {
                                            view! {cx,
                                                <div
                                                    class="action-icon"
                                                    on:click=move |_| { expect_context::<CrudInstanceContext<T>>(cx).read(entity_clone4.as_ref().clone()) }
                                                >
                                                    <Icon icon=BsIcon::BsEye/>
                                                </div>
                                            }.into_view(cx)
                                        } else {
                                            ().into_view(cx)
                                        }
                                    }
                                    {
                                        if edit_allowed.get() {
                                            view! {cx,
                                                <div
                                                    class="action-icon"
                                                    on:click=move |_| { expect_context::<CrudInstanceContext<T>>(cx).edit(entity_clone5.as_ref().clone().into()) }
                                                >
                                                    <Icon icon=BsIcon::BsPencil/>
                                                </div>
                                            }.into_view(cx)
                                        } else {
                                            ().into_view(cx)
                                        }
                                    }
                                    {
                                        if delete_allowed.get() {
                                            view! {cx,
                                                <div
                                                    class="action-icon"
                                                    // TODO: This is really an action. Implement this. SET MODAL!
                                                    //on:click=move |_| { expect_context::<CrudInstanceContext<T>>(cx).edit(entity.as_ref().clone()) }
                                                >
                                                    <Icon icon=BsIcon::BsTrash/>
                                                </div>
                                            }.into_view(cx)
                                        } else {
                                            ().into_view(cx)
                                        }
                                    }

                                    //{
                                    //    ctx.props().additional_item_actions.iter().map(|action| {
                                    //        // TODO: can we eliminate some clone()'s?
                                    //        let cloned_action = action.clone();
                                    //        let cloned_entity = entity.clone();
                                    //        html! {
                                    //            <div
                                    //                class={"action-icon"}
                                    //                onclick={link.callback(move |_| Msg::ActionTriggered((cloned_action.clone(), cloned_entity.clone())))}>
                                    //                <CrudIcon variant={action.get_icon().unwrap_or(Bi::Question)}/>
                                    //            </div>
                                    //        }
                                    //    }).collect::<Html>()
                                    //}
                                </div>
                            </td>
                        }.into_view(cx),
                        false => ().into_view(cx)
                    }
                }}
            </tr>
        }
    };

    // TODO: Extract to leptonic
    view! {cx,
        <div class={"crud-table-wrapper"}>
            <table class={"crud-table crud-table-bordered crud-table-hoverable"}>
                // Header
                <CrudTableHeaderL
                    _phantom={PhantomData::<T>::default()}
                    headers=headers
                    with_actions=with_actions
                    with_select_column=list_ctx.has_data
                    all_selected=list_ctx.all_selected
                />

                // Body
                <tbody>
                    {move || match list_ctx.data.get() {
                        Some(data) => match data.len() {
                            0 => view! {cx,
                                <tr>
                                    <td colspan="100%" class="no-data">
                                        {"Keine Daten"}
                                    </td>
                                </tr>
                            }
                            .into_view(cx),
                            _ => view! {cx,
                                <For
                                    each=move || data.as_ref().clone() // TODO: Performance? Remove Rc in type?
                                    key=|_entity| Uuid::new_v4() // TODO: Use entity!
                                    view=render_entry
                                />
                            }
                            .into_view(cx),
                        },
                        None => view! {cx,
                            <tr>
                                <td colspan="100%">
                                    {"\u{00a0}"} // nbsp, see https://doc.rust-lang.org/std/primitive.char.html
                                </td>
                            </tr>
                        }
                        .into_view(cx),
                    }}
                </tbody>

                // Footer
                <CrudTableFooterL />
            </table>
        </div>
    }
}
