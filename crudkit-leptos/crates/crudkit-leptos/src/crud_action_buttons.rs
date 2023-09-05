use crudkit_web::CrudMainTrait;
use leptonic::prelude::*;
use leptos::*;

use crate::{
    crud_action::EntityModalGeneration,
    crud_instance::CrudInstanceContext,
    prelude::{CrudActionContext, CrudEntityAction, States},
};

#[component]
pub fn CrudActionButtons<T>(
    action_ctx: CrudActionContext<T>,
    #[prop(into)] actions: Signal<Vec<CrudEntityAction<T>>>,
    #[prop(into)] input: Signal<Option<T::UpdateModel>>,
    required_state: States,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let instance_ctx = expect_context::<CrudInstanceContext<T>>();
    view! {
        <For
            each=move || actions.get()
            key=|action| match action {
                CrudEntityAction::Custom {id, name: _, icon: _, button_color: _, valid_in: _, action: _, modal: _} => *id
            }
            view=move | action| match action {
                CrudEntityAction::Custom {id, name, icon, button_color, valid_in, action, modal} => {
                    valid_in.contains(&required_state).then(|| {
                        if let Some(modal_generator) = modal {
                            view! {
                                <Button
                                    color=button_color
                                    disabled=Signal::derive( move || action_ctx.is_action_executing(id))
                                    on_click=move |_| action_ctx.request_action(id)
                                >
                                    { icon.map(|icon| view! { <Icon icon=icon/>}) }
                                    { name.clone() }
                                </Button>
                                {
                                    modal_generator.call(EntityModalGeneration {
                                        show_when: Signal::derive( move || action_ctx.is_action_requested(id)),
                                        state: input.into(),
                                        cancel: create_callback( move |_| action_ctx.cancel_action(id)),
                                        execute: create_callback( move |action_payload| action_ctx.trigger_entity_action(id, input.get().unwrap(), action_payload, action, instance_ctx)),
                                    })
                                }
                            }.into_view()
                        } else {
                            view! {
                                <Button
                                    color=button_color
                                    disabled=Signal::derive( move || action_ctx.is_action_executing(id))
                                    on_click=move |_| action_ctx.trigger_entity_action(id, input.get().unwrap(), None, action, instance_ctx)
                                >
                                    { icon.map(|icon| view! { <Icon icon=icon/>}) }
                                    { name.clone() }
                                </Button>
                            }.into_view()
                        }
                    })
                }
            }
        />
    }
}
