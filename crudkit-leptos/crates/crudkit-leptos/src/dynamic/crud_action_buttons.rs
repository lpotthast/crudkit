use crate::dynamic::crud_action::{CrudEntityAction, EntityActionViewInput, States};
use crate::dynamic::crud_action_context::CrudActionContext;
use crate::dynamic::crud_instance::CrudInstanceContext;
use crudkit_web::dynamic::AnyUpdateModel;
use leptonic::components::prelude::*;
use leptos::prelude::*;

#[component]
pub fn CrudActionButtons(
    action_ctx: CrudActionContext,
    #[prop(into)] actions: Signal<Vec<CrudEntityAction>>,
    #[prop(into)] input: Signal<Option<AnyUpdateModel>>,
    required_state: States,
) -> impl IntoView {
    let instance_ctx = expect_context::<CrudInstanceContext>();
    view! {
        <For
            each=move || actions.get()
            key=|action| action.id
            children=move |CrudEntityAction { id, name, icon, button_color, valid_in, action, view }| {
                let action_clone = action.clone();
                valid_in
                    .contains(&required_state)
                    .then(|| {
                        if let Some(view_generator) = view {
                            view! {
                                <Button
                                    color=button_color
                                    disabled=Signal::derive(move || { action_ctx.is_action_executing(id) })
                                    on_press=move |_| action_ctx.request_action(id)
                                >
                                    {icon.map(|icon| view! { <Icon icon=icon/> })}
                                    {name.clone()}
                                </Button>

                                {view_generator
                                    .run(EntityActionViewInput {
                                        show_when: Signal::derive(move || { action_ctx.is_action_requested(id) }),
                                        state: input.into(),
                                        cancel: Callback::new(move |_| { action_ctx.cancel_action(id) }),
                                        execute: Callback::new(move |action_payload| {
                                            action_ctx
                                                .trigger_entity_action(
                                                    id,
                                                    input.get().unwrap(),
                                                    action_payload,
                                                    action.clone(),
                                                    instance_ctx,
                                                )
                                        }),
                                    })}
                            }
                                .into_any()
                        } else {
                            view! {
                                <Button
                                    color=button_color
                                    disabled=Signal::derive(move || { action_ctx.is_action_executing(id) })
                                    on_press=move |_| {
                                        action_ctx
                                            .trigger_entity_action(
                                                id,
                                                input.get().unwrap(),
                                                None,
                                                action_clone.clone(),
                                                instance_ctx,
                                            )
                                    }
                                >

                                    {icon.map(|icon| view! { <Icon icon=icon/> })}
                                    {name.clone()}
                                </Button>
                            }
                                .into_any()
                        }
                    })
            }
        />
    }
}
