use crate::crud_field::CrudField;
use crate::crud_instance_config::FieldRendererRegistry;
use crate::ReactiveValue;
use crudkit_core::Value;
use crudkit_web::prelude::*;
use crudkit_web::{FieldMode, TabId};
use leptonic::components::prelude::*;
use leptos::prelude::*;
use std::collections::HashMap;

#[component]
pub fn CrudFields<F: DynField>(
    field_renderer_registry: Signal<FieldRendererRegistry<F>>,
    #[prop(into)] elements: Signal<Vec<Elem<F>>>,
    #[prop(into)] signals: StoredValue<HashMap<F, ReactiveValue>>,
    mode: FieldMode,
    value_changed: Callback<(F, Result<Value, String>)>,
    on_tab_selection: Callback<TabId>,
) -> impl IntoView {
    move || {
        elements
            .get()
            .into_iter()
            .map(|elem| {
                match elem {
                    Elem::Enclosing(enclosing) => {
                        match enclosing {
                            Enclosing::None(group) => view! {
                                <CrudFields
                                    field_renderer_registry=field_renderer_registry
                                    elements=group.children.clone()
                                    signals=signals
                                    mode=mode.clone()
                                    value_changed=value_changed
                                    on_tab_selection=on_tab_selection
                                />
                            }
                                .into_any(),
                            Enclosing::Tabs(tabs) => view! {
                                <Tabs>
                                    // active_tab={ctx.props().active_tab.clone()}
                                    // on_tab_selection={ctx.link().callback(|label| Msg::TabSelected(label))}
                                    {
                                        tabs.into_iter().map(move |tab| {
                                            let id = tab.id.clone();
                                            view! {
                                                <Tab
                                                    name=tab.id
                                                    label=move || tab.label.name.clone()
                                                    on_show=move |()| {
                                                        on_tab_selection.run(id.clone())
                                                    }
                                                >
                                                    <CrudFields
                                                        field_renderer_registry=field_renderer_registry
                                                        elements=tab.group.children.clone()
                                                        signals=signals
                                                        mode=mode.clone()
                                                        value_changed=value_changed
                                                        on_tab_selection=on_tab_selection
                                                    />
                                                </Tab>
                                            }
                                        }).collect_view()
                                    }
                                </Tabs>
                            }
                                .into_any(),
                            Enclosing::Card(group) => view! {
                                <Card>
                                    <CrudFields
                                        field_renderer_registry=field_renderer_registry
                                        elements=group.children.clone()
                                        signals=signals
                                        mode=mode.clone()
                                        value_changed=value_changed
                                        on_tab_selection=on_tab_selection
                                    />
                                </Card>
                            }
                                .into_any(),
                        }
                    }
                    Elem::Field((field, field_options)) => view! {
                        <CrudField
                            field_renderer_registry=field_renderer_registry
                            field=field.clone()
                            field_options=field_options.clone()
                            field_mode=mode.clone()
                            signals=signals
                            value=signals.with_value(|map| {
                                match map.get(&field) {
                                    Some(value) => *value,
                                    None => panic!("Signal map to contain signal for field: {}", field.get_name()),
                                }
                            })
                            value_changed=value_changed
                        />
                    }
                        .into_any(),
                    Elem::Separator => view! { <Separator/> }.into_any(),
                }
            })
            .collect_view()
    }
}
