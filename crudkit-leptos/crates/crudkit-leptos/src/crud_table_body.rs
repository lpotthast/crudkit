use std::{collections::HashMap, marker::PhantomData, rc::Rc};

use crudkit_web::{
    CrudDataTrait, CrudFieldNameTrait, CrudFieldValueTrait, CrudIdTrait,
    CrudMainTrait, CrudSimpleView, DeletableModel, FieldMode, FieldOptions, HeaderOptions,
};
use leptonic::prelude::*;
use leptonic::components::prelude::*;
use leptos::*;

use crate::{
    crud_action::CrudActionTrait, crud_field::CrudField, crud_instance::CrudInstanceContext,
    crud_instance_config::DynSelectConfig, crud_list_view::CrudListViewContext,
    crud_table::NoDataAvailable, IntoReactiveValue, prelude::CustomFields, ReactiveValue,
};

#[component]
pub fn CrudTableBody<T>(
    _phantom: PhantomData<T>,
    #[prop(into)] data: Signal<Result<Rc<Vec<T::ReadModel>>, NoDataAvailable>>,
    #[prop(into)] api_base_url: Signal<String>,
    #[prop(into)] headers: Signal<Vec<(<T::ReadModel as CrudDataTrait>::Field, HeaderOptions)>>,
    #[prop(into)] custom_fields: Signal<CustomFields<T::ReadModel, leptos::View>>,
    #[prop(into)] field_config: Signal<
        HashMap<<T::ReadModel as CrudDataTrait>::Field, DynSelectConfig>,
    >,
    #[prop(into)] read_allowed: Signal<bool>,
    #[prop(into)] edit_allowed: Signal<bool>,
    #[prop(into)] delete_allowed: Signal<bool>,
    #[prop(into)] additional_item_actions: Signal<Vec<Rc<Box<dyn CrudActionTrait>>>>,
) -> impl IntoView
where
    T: CrudMainTrait + 'static,
{
    let render_entry = move |entity: T::ReadModel| {
        // A signal map is required to render custom fields,
        // as each custom field may want to access values of other fields to determine its redering output or processing!
        let signals: StoredValue<HashMap<<T::ReadModel as CrudDataTrait>::Field, ReactiveValue>> =
        store_value({
            let mut map = HashMap::new();
            for field in T::ReadModel::get_all_fields() {
                let initial = field.get_value(&entity);
                map.insert(field, initial.into_reactive_value());
            }
            map
        });


        // TODO: Check https://github.com/rust-lang/rfcs/issues/2407, we might be able to remove explicit clones in the future!
        let stored_entity: ReadSignal<<T as CrudMainTrait>::ReadModel> =
            create_rw_signal(entity).read_only(); // TODO: Move signal creation up

        let with_actions = Signal::derive(move || {
            !additional_item_actions.get().is_empty()
                || read_allowed.get()
                || edit_allowed.get()
                || delete_allowed.get()
        });

        let instance_ctx = expect_context::<CrudInstanceContext<T>>();
        let list_ctx = expect_context::<CrudListViewContext<T>>();
        let is_selected = create_memo(move |_prev| {
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

        let read = move |entity: T::ReadModel| instance_ctx.read(entity.get_id());
        let edit = move |entity: T::ReadModel| instance_ctx.edit(entity.into().get_id());
        let delete = move |entity: T::ReadModel| {
            instance_ctx.request_deletion_of(DeletableModel::Read(entity))
        };
        let trigger_action =
            move |entity: T::ReadModel, action: Rc<Box<dyn CrudActionTrait>>| todo!();

        let dummy_value_changed_callback = Callback::new(move |_| {});

        view! {
            <tr class="interactable" on:click=move |_e| { instance_ctx.edit(stored_entity.get().into().get_id()) }>
                <td class="select fit-content" on:click=move |e| e.stop_propagation()>
                    <Checkbox checked=is_selected set_checked=move |checked| {
                        if checked != is_selected.get_untracked() {
                            toggle_selected()
                        }
                    }/>
                </td>

                <For
                    each=move || headers.get()
                    key=|(field, _options)| field.get_name()
                    children=move |(field, options)| {
                        let entity = stored_entity.get();
                        let reactive_value = {
                            let initial = field.get_value(&entity);
                            initial.into_reactive_value()
                        };
                        view! {
                            // TODO: Is it ok to recreate this reactive value on the fly?
                            // TODO: Optimize. Do we still need a StoredEntity?

                            <td class:fit-content=options.min_width>
                                <CrudField
                                    // children={ctx.props().children.clone()} // TODO: make this work
                                    custom_fields=custom_fields
                                    field_config=field_config
                                    api_base_url=api_base_url
                                    current_view=CrudSimpleView::List
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
                            </td>
                        }
                    }
                />

                {move || {
                    with_actions
                        .get()
                        .then(|| {
                            view! {
                                <td class="fit-content" on:click=|e| e.stop_propagation()>
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
                                        {edit_allowed
                                            .get()
                                            .then(|| {
                                                view! {
                                                    <div
                                                        class="action-icon"
                                                        on:click=move |_| edit(stored_entity.get())
                                                    >
                                                        <Icon icon=icondata::BsPencil/>
                                                    </div>
                                                }
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
                                            key=|entity| entity.get_name()
                                            children=move |action| {
                                                let icon = action.get_icon().unwrap_or(icondata::BsQuestion);
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
                                </td>
                            }
                        })
                }}

            </tr>
        }
    };

    view! {
        <tbody>
            {move || match data.get() {
                Ok(data) => {
                    match data.len() {
                        0 => {
                            view! {
                                <tr>
                                    <td colspan="100%" class="no-data">
                                        {"Keine Daten"}
                                    </td>
                                </tr>
                            }.into_view()
                        }
                        _ => {
                            view! {
                                <For
                                    each=move || data.as_ref().clone()
                                    key=|entity| entity.get_id()
                                    children=render_entry
                                />
                            }.into_view()
                        }
                    }
                }
                Err(no_data) => {
                    match no_data {
                        NoDataAvailable::NotYetLoaded => {
                            view! {
                                <tr>
                                    <td colspan="100%">" "</td>
                                </tr>
                            }.into_view()
                        }
                        NoDataAvailable::RequestFailed(reason) => {
                            view! {
                                <tr>
                                    <td colspan="100%">{format!("No data available: {reason:?}")}</td>
                                </tr>
                            }.into_view()
                        }
                        NoDataAvailable::RequestReturnedNoData(reason) => {
                            view! {
                                <tr>
                                    <td colspan="100%">{format!("No data available: {reason:?}")}</td>
                                </tr>
                            }.into_view()
                        }
                    }
                }
            }}
        </tbody>
    }
}
