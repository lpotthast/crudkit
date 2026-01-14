use crate::ReactiveValue;
use crate::generic::crud_field::CrudField;
use crate::generic::custom_field::CustomFields;
use crudkit_core::Value;
use crudkit_web::generic::prelude::*;
use leptonic::components::prelude::*;
use leptos::prelude::*;
use std::collections::HashMap;

#[component]
pub fn CrudFields<T>(
    custom_fields: Signal<CustomFields<T>>,
    #[prop(into)] elements: Signal<Vec<Elem<T>>>,
    #[prop(into)] signals: StoredValue<HashMap<T::Field, ReactiveValue>>,
    mode: FieldMode,
    current_view: CrudSimpleView,
    value_changed: Callback<(T::Field, Result<Value, String>)>,
    on_tab_selection: Callback<TabId>,
) -> impl IntoView
where
    T: CrudDataTrait + 'static,
{
    move || {
        elements
            .get()
            .into_iter()
            .map(|elem| {
                let on_tab_selection = on_tab_selection.clone();
                match elem {
                    Elem::Enclosing(enclosing) => {
                        match enclosing {
                            Enclosing::None(group) => view! {
                                <CrudFields
                                    custom_fields=custom_fields
                                    elements=group.children.clone()
                                    signals=signals
                                    mode=mode.clone()
                                    current_view=current_view
                                    value_changed=value_changed
                                    on_tab_selection=on_tab_selection.clone()
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
                                            let on_tab_selection1 = on_tab_selection.clone();
                                            let on_tab_selection2 = on_tab_selection.clone();
                                            view! {
                                                <Tab
                                                    name=tab.id
                                                    label=move || tab.label.name.clone()
                                                    on_show=move |()| {
                                                        on_tab_selection1.run(id.clone())
                                                    }
                                                >
                                                    <CrudFields
                                                        custom_fields=custom_fields
                                                        elements=tab.group.children.clone()
                                                        signals=signals
                                                        mode=mode.clone()
                                                        current_view=current_view
                                                        value_changed=value_changed
                                                        on_tab_selection=on_tab_selection2.clone()
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
                                        custom_fields=custom_fields
                                        elements=group.children.clone()
                                        signals=signals
                                        mode=mode.clone()
                                        current_view=current_view
                                        value_changed=value_changed
                                        on_tab_selection=on_tab_selection.clone()
                                    />
                                </Card>
                            }
                            .into_any(),
                        }
                    }
                    Elem::Field((field, field_options)) => view! {
                        <CrudField
                            custom_fields=custom_fields
                            current_view=current_view
                            field=field.clone()
                            field_options=field_options.clone()
                            field_mode=mode.clone()
                            signals=signals
                            value=signals.with_value(|map| {
                                *map.get(&field).expect("Signal map to contain signal for field")
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
