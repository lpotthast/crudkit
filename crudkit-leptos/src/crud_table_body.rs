use crate::crud_action::CrudActionTrait;
use crate::crud_field::CrudField;
use crate::crud_instance::CrudInstanceContext;
use crate::crud_instance_config::{FieldRendererRegistry, Header};
use crate::crud_list_view::CrudListViewContext;
use crate::crud_table::NoDataAvailable;
use crudkit_web::prelude::*;
use crudkit_web::{FieldMode, FieldOptions};
use leptonic::components::prelude::*;
use leptonic::prelude::*;
use leptos::prelude::*;
use std::sync::Arc;

#[component]
pub fn CrudTableBody(
    #[prop(into)] data: Signal<Result<Arc<Vec<DynReadModel>>, NoDataAvailable>>,
    #[prop(into)] headers: Signal<Vec<Header>>,
    #[prop(into)] field_renderer_registry: Signal<FieldRendererRegistry<DynReadField>>,
    #[prop(into)] read_allowed: Signal<bool>,
    #[prop(into)] edit_allowed: Signal<bool>,
    #[prop(into)] delete_allowed: Signal<bool>,
    #[prop(into)] additional_item_actions: Signal<Vec<Arc<Box<dyn CrudActionTrait>>>>, // TODO: Use AnyAction
) -> impl IntoView {
    let ctx = expect_context::<CrudInstanceContext>();

    let render_entry = move |entity: DynReadModel| {
        // A signal map is required to render custom fields,
        // as each custom field may want to access values of other fields to determine its rendering output or processing!
        let signals = StoredValue::new(
            ctx.static_config
                .read_value()
                .model_handler
                .read_model_to_signal_map
                .run(entity.clone()),
        ); // TODO: Can we get rid of this clone???

        // TODO: Check https://github.com/rust-lang/rfcs/issues/2407, we might be able to remove explicit clones in the future!
        let stored_entity: ReadSignal<DynReadModel> = RwSignal::new(entity).read_only(); // TODO: Move signal creation up

        let with_actions = Signal::derive(move || {
            !additional_item_actions.get().is_empty()
                || read_allowed.get()
                || edit_allowed.get()
                || delete_allowed.get()
        });

        let instance_ctx = expect_context::<CrudInstanceContext>();
        let list_ctx = expect_context::<CrudListViewContext>();
        let is_selected = Memo::new(move |_prev| {
            stored_entity.with(|stored_entity| {
                list_ctx
                    .selected
                    .get()
                    .iter()
                    .find(|it| *it == stored_entity)
                    .is_some()
            })
        });
        let toggle_selected = move || list_ctx.toggle_entity_selection(stored_entity.get());

        // TODO: These closures are now identical...
        let read = move |entity: DynReadModel| instance_ctx.read(entity.id());
        let edit = move |entity: DynReadModel| instance_ctx.edit(entity.id());
        let delete = move |entity: DynReadModel| {
            instance_ctx.request_deletion_of(DynReadOrUpdateModel::Read(entity))
        };
        // TODO: why is this Arc<Box<...>>?
        let trigger_action =
            move |_entity: DynReadModel, _action: Arc<Box<dyn CrudActionTrait>>| todo!();

        let dummy_value_changed_callback = Callback::new(move |_| {});

        view! {
            <TableRow attr:class="interactable" on:click=move |_e| { instance_ctx.edit(stored_entity.get().id()) }>
                <TableCell attr:class="select fit-content" on:click=move |e| e.stop_propagation()>
                    <Checkbox checked=is_selected set_checked=move |checked| {
                        if checked != is_selected.get_untracked() {
                            toggle_selected()
                        }
                    }/>
                </TableCell>

                <For
                    each=move || headers.get()
                    key=|header| header.field.name()
                    children=move |Header { field, options }| {
                        // TODO: Why is this access to stored_entity necessary?
                        let _ = stored_entity.get();
                        let reactive_value = *signals.read_value().get(&field).unwrap_or_else(|| panic!("Expected reactive value for field '{}'", field.name()));
                        view! {
                            // TODO: Is it ok to recreate this reactive value on the fly?
                            // TODO: Optimize. Do we still need a StoredEntity?

                            <TableCell class:fit-content=options.min_width>
                                <CrudField
                                    field_renderer_registry=field_renderer_registry
                                    field=field.clone()
                                    field_options=FieldOptions {
                                        disabled: false,
                                        label: None,
                                        date_time_display: options.date_time_display,
                                    }
                                    // TODO: We could tie the value_changed callback to the field_mode, as it is only required when a value can actually change!
                                    field_mode=FieldMode::Display
                                    signals=signals
                                    value=reactive_value
                                    value_changed=dummy_value_changed_callback.clone()
                                />
                            </TableCell>
                        }
                    }
                />

                {move || {
                    with_actions
                        .get()
                        .then(|| {
                            view! {
                                <TableCell attr:class="fit-content" on:click=|e| e.stop_propagation()>
                                    <div class="action-icons">
                                        {read_allowed
                                            .get()
                                            .then(|| {
                                                view! {
                                                    <div
                                                        class="action-icon"
                                                        on:click=move |_| read(stored_entity.get())
                                                    >
                                                        <Icon icon=icondata::BsEye/>
                                                    </div>
                                                }
                                            })}
                                        {edit_allowed.get().then(|| view! {
                                            <div
                                                class="action-icon"
                                                on:click=move |_| edit(stored_entity.get())
                                            >
                                                <Icon icon=icondata::BsPencil/>
                                            </div>
                                        })}
                                        {delete_allowed
                                            .get()
                                            .then(|| {
                                                view! {
                                                    <div
                                                        class="action-icon"
                                                        on:click=move |_| delete(stored_entity.get())
                                                    >
                                                        <Icon icon=icondata::BsTrash/>
                                                    </div>
                                                }
                                            })}
                                        <For
                                            each=move || additional_item_actions.get()
                                            key=|action| action.name()
                                            children=move |action| {
                                                let icon = action.icon().unwrap_or(icondata::BsQuestion);
                                                view! {
                                                    <div
                                                        class="action-icon"
                                                        on:click=move |_| trigger_action(
                                                            stored_entity.get(),
                                                            action.clone(),
                                                        )
                                                    >
                                                        <Icon icon=icon/>
                                                    </div>
                                                }
                                            }
                                        />

                                    </div>
                                </TableCell>
                            }
                        })
                }}
            </TableRow>
        }
    };

    view! {
        <TableBody>
            {move || match data.get() {
                Ok(data) => {
                    match data.len() {
                        0 => {
                            view! {
                                <TableRow>
                                    <TableCell attr:colspan="100%" attr:class="no-data">
                                        {"Keine Daten"}
                                    </TableCell>
                                </TableRow>
                            }.into_any()
                        }
                        _ => {
                            view! {
                                <For
                                    each=move || data.as_ref().clone()
                                    key=|entity| entity.id()
                                    children=render_entry
                                />
                            }.into_any()
                        }
                    }
                },
                Err(no_data) => {
                    match no_data {
                        NoDataAvailable::NotYetLoaded => {
                            view! {
                                <TableRow>
                                    <TableCell attr:colspan="100%">" "</TableCell>
                                </TableRow>
                            }.into_any()
                        }
                        NoDataAvailable::RequestFailed(reason) => {
                            view! {
                                <TableRow>
                                    <TableCell attr:colspan="100%">{format!("No data available: {reason:?}")}</TableCell>
                                </TableRow>
                            }.into_any()
                        }
                        NoDataAvailable::RequestReturnedNoData(reason) => {
                            view! {
                                <TableRow>
                                    <TableCell attr:colspan="100%">{format!("No data available: {reason:?}")}</TableCell>
                                </TableRow>
                            }.into_any()
                        }
                    }
                },
            }}
        </TableBody>
    }
}
