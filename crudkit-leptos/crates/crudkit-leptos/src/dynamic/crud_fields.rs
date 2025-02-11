use crate::dynamic::crud_field::CrudField;
use crate::dynamic::custom_field::CustomFields;
use crate::shared::crud_instance_config::DynSelectConfig;
use crate::ReactiveValue;
use crudkit_web::dynamic::prelude::*;
use leptonic::components::prelude::*;
use leptos::prelude::*;
use std::collections::HashMap;
// TODO: Propagate tab selection...

#[component]
pub fn CrudFields(
    custom_fields: Signal<CustomFields>,
    field_config: Signal<HashMap<AnyField, DynSelectConfig>>,
    #[prop(into)] elements: Signal<Vec<AnyElem>>,
    #[prop(into)] signals: StoredValue<HashMap<AnyField, ReactiveValue>>,
    mode: FieldMode,
    current_view: CrudSimpleView,
    value_changed: Callback<(AnyField, Result<Value, String>)>,
    // active_tab: Option<Label>,
    on_tab_selection: Callback<(TabId,)>,
) -> impl IntoView {
    move || {
        elements
            .get()
            .into_iter()
            .map(|elem| {
                let on_tab_selection = on_tab_selection.clone();
                match elem {
                    AnyElem::Enclosing(enclosing) => {
                        match enclosing {
                            AnyEnclosing::None(group) => view! {
                                <CrudFields
                                    custom_fields=custom_fields
                                    field_config=field_config
                                    elements=group.children.clone()
                                    signals=signals
                                    mode=mode.clone()
                                    current_view=current_view
                                    value_changed=value_changed
                                    // active_tab={ctx.props().active_tab.clone()}
                                    on_tab_selection=on_tab_selection.clone()
                                />
                            }
                            .into_any(),
                            AnyEnclosing::Tabs(tabs) => view! {
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
                                                        on_tab_selection1.run((id.clone(),))
                                                    }
                                                >
                                                    <CrudFields
                                                        custom_fields=custom_fields
                                                        field_config=field_config
                                                        elements=tab.group.children.clone()
                                                        signals=signals
                                                        mode=mode.clone()
                                                        current_view=current_view
                                                        value_changed=value_changed
                                                        // active_tab={ctx.props().active_tab.clone()}
                                                        on_tab_selection=on_tab_selection2.clone()
                                                    />
                                                </Tab>
                                            }
                                        }).collect_view()
                                    }
                                </Tabs>
                            }
                            .into_any(),
                            AnyEnclosing::Card(group) => view! {
                                <Card>
                                    <CrudFields
                                        custom_fields=custom_fields
                                        field_config=field_config
                                        elements=group.children.clone()
                                        signals=signals
                                        mode=mode.clone()
                                        current_view=current_view
                                        value_changed=value_changed
                                        // active_tab={ctx.props().active_tab.clone()}
                                        on_tab_selection=on_tab_selection.clone()
                                    />
                                </Card>
                            }
                            .into_any(),
                        }
                    }
                    AnyElem::Field((field, field_options)) => view! {
                        <CrudField
                            custom_fields=custom_fields
                            field_config=field_config
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
                    AnyElem::Separator => view! { <Separator/> }.into_any(),
                }
            })
            .collect_view()
    }
}
