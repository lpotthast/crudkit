use crate::crud_action::{CrudEntityAction, EntityActionViewInput, States};
use crate::crud_action_context::CrudActionContext;
use crate::crud_instance::CrudInstanceContext;
use crudkit_web::prelude::DynUpdateModel;
use leptonic::components::prelude::*;
use leptos::prelude::*;

#[component]
pub fn CrudActionButtons(
    action_ctx: CrudActionContext,
    #[prop(into)] actions: Signal<Vec<CrudEntityAction>>,
    #[prop(into)] input: Signal<Option<DynUpdateModel>>,
    required_state: States,
) -> impl IntoView {
    let instance_ctx = expect_context::<CrudInstanceContext>();
    view! {
        <For
            each=move || actions.get()
            key=|action| action.id
            children=move |CrudEntityAction { id, name, icon, button_color, valid_in, action, view }| {
                let action_clone = action;
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
                                        state: input,
                                        cancel: Callback::new(move |_| { action_ctx.cancel_action(id) }),
                                        execute: Callback::new(move |action_payload| {
                                            // Guard against None; button should be disabled when input is None.
                                            let Some(model) = input.get() else { return };
                                            action_ctx
                                                .trigger_entity_action(
                                                    id,
                                                    model,
                                                    action_payload,
                                                    action,
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
                                        // Guard against None; button should be disabled when input is None.
                                        let Some(model) = input.get() else { return };
                                        action_ctx
                                            .trigger_entity_action(
                                                id,
                                                model,
                                                None,
                                                action_clone,
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
