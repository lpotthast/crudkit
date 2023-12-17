use std::collections::HashMap;

use crudkit_web::{CrudDataTrait, CrudSimpleView, Elem, Enclosing, FieldMode, TabId, Value};
use leptonic::prelude::*;
use leptos::*;

use crate::{
    crud_field::CrudField, crud_instance_config::DynSelectConfig, prelude::CustomFields,
    ReactiveValue,
};

// TODO: Propagate tab selection...

#[component]
pub fn CrudFields<T>(
    // children: ChildrenRenderer<Item>,
    custom_fields: Signal<CustomFields<T, leptos::View>>,
    field_config: Signal<HashMap<T::Field, DynSelectConfig>>,
    api_base_url: Signal<String>,
    #[prop(into)] elements: MaybeSignal<Vec<Elem<T>>>,
    #[prop(into)] signals: StoredValue<HashMap<T::Field, ReactiveValue>>,
    mode: FieldMode,
    current_view: CrudSimpleView,
    value_changed: Callback<(T::Field, Result<Value, String>)>,
    // active_tab: Option<Label>,
    on_tab_selection: Callback<TabId>,
    entity: Signal<T>,
) -> impl IntoView
where
    T: CrudDataTrait + 'static,
{
    move || {
        elements
            .get()
            .into_iter()
            .map(|elem| {
                let value_changed = value_changed.clone();
                let on_tab_selection = on_tab_selection.clone();
                match elem {
                    Elem::Enclosing(enclosing) => {
                        match enclosing {
                            Enclosing::None(group) => view! {
                                <CrudFields
                                    custom_fields=custom_fields
                                    field_config=field_config
                                    api_base_url=api_base_url
                                    elements=group.children.clone()
                                    signals=signals.clone()
                                    mode=mode.clone()
                                    current_view=current_view.clone()
                                    value_changed=value_changed.clone()
                                    // active_tab={ctx.props().active_tab.clone()}
                                    on_tab_selection=on_tab_selection.clone()
                                    entity=entity
                                />
                            }
                            .into_view(),
                            Enclosing::Tabs(tabs) => view! {
                                <Tabs>
                                    // active_tab={ctx.props().active_tab.clone()}
                                    // on_tab_selection={ctx.link().callback(|label| Msg::TabSelected(label))}
                                    {
                                        tabs.into_iter().map(move |tab| {
                                            let id = tab.id.clone();
                                            let signals = signals.clone();
                                            let value_changed = value_changed.clone();
                                            let on_tab_selection1 = on_tab_selection.clone();
                                            let on_tab_selection2 = on_tab_selection.clone();
                                            view! {
                                                <Tab
                                                    name=tab.id
                                                    label=tab.label.name.clone().into_view()
                                                    on_show=move |()| {
                                                        on_tab_selection1.call(id.clone())
                                                    }
                                                >
                                                    <CrudFields
                                                        custom_fields=custom_fields
                                                        field_config=field_config
                                                        api_base_url=api_base_url
                                                        elements=tab.group.children.clone()
                                                        signals=signals.clone()
                                                        mode=mode.clone()
                                                        current_view=current_view.clone()
                                                        value_changed=value_changed.clone()
                                                        // active_tab={ctx.props().active_tab.clone()}
                                                        on_tab_selection=on_tab_selection2.clone()
                                                        entity=entity
                                                    />
                                                </Tab>
                                            }
                                        })
                                        .collect_view()}
                                </Tabs>
                            }
                            .into_view(),
                            Enclosing::Card(group) => view! {
                                <div class="crud-card">
                                    <CrudFields
                                        custom_fields=custom_fields
                                        field_config=field_config
                                        api_base_url=api_base_url
                                        elements=group.children.clone()
                                        signals=signals.clone()
                                        mode=mode.clone()
                                        current_view=current_view.clone()
                                        value_changed=value_changed.clone()
                                        // active_tab={ctx.props().active_tab.clone()}
                                        on_tab_selection=on_tab_selection.clone()
                                        entity=entity
                                    />
                                </div>
                            }
                            .into_view(),
                        }
                    }
                    Elem::Field((field, field_options)) => view! {
                        <CrudField
                            custom_fields=custom_fields
                            field_config=field_config
                            api_base_url=api_base_url
                            current_view=current_view
                            field=field.clone()
                            field_options=field_options.clone()
                            field_mode=mode.clone()
                            signals=signals
                            value=signals.with_value(|map| {
                                *map.get(&field).expect("Signal map to contain signal for field")
                            })
                            value_changed=value_changed.clone()
                            entity=entity
                        />
                    }
                    .into_view(),
                    Elem::Separator => view! { <Separator/> }.into_view(),
                }
            })
            .collect_view()
    }
}
