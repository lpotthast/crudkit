use crate::dynamic::crud_field::CrudField;
use crate::dynamic::crud_instance_config::UpdateElements;
use crate::dynamic::custom_field::{CustomCreateFields, CustomUpdateFields};
use crate::shared::crud_instance_config::DynSelectConfig;
use crate::ReactiveValue;
use crudkit_web::dynamic::prelude::*;
use crudkit_web::dynamic::{AnyCreateField, AnyUpdateField};
use leptonic::components::prelude::*;
use leptos::prelude::*;
use std::collections::HashMap;

// TODO: Propagate tab selection...
#[component]
pub fn CrudCreateFields(
    custom_fields: Signal<CustomCreateFields>,
    field_config: Signal<HashMap<AnyCreateField, DynSelectConfig>>,
    #[prop(into)] elements: Signal<Vec<Elem<AnyCreateField>>>,
    #[prop(into)] signals: StoredValue<HashMap<AnyCreateField, ReactiveValue>>,
    mode: FieldMode,
    current_view: CrudSimpleView,
    value_changed: Callback<(AnyCreateField, Result<Value, String>)>,
    // active_tab: Option<Label>,
    on_tab_selection: Callback<TabId>,
) -> impl IntoView {
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
                                <CrudCreateFields
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
                                                    <CrudCreateFields
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
                            Enclosing::Card(group) => view! {
                                <Card>
                                    <CrudCreateFields
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
                    Elem::Field((field, field_options)) => view! {
                        <CrudField
                            custom_fields=custom_fields
                            field_config=field_config
                            current_view=current_view
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

// TODO: Propagate tab selection...
#[component]
pub fn CrudUpdateFields(
    custom_fields: Signal<CustomUpdateFields>,
    field_config: Signal<HashMap<AnyUpdateField, DynSelectConfig>>,
    #[prop(into)] elements: Signal<UpdateElements>,
    #[prop(into)] signals: StoredValue<HashMap<AnyUpdateField, ReactiveValue>>,
    mode: FieldMode,
    current_view: CrudSimpleView,
    value_changed: Callback<(AnyUpdateField, Result<Value, String>)>,
    // active_tab: Option<Label>,
    on_tab_selection: Callback<TabId>,
) -> impl IntoView {
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
                                <CrudUpdateFields
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
                                                    <CrudUpdateFields
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
                            Enclosing::Card(group) => view! {
                                <Card>
                                    <CrudUpdateFields
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
                    Elem::Field((field, field_options)) => view! {
                        <CrudField
                            custom_fields=custom_fields
                            field_config=field_config
                            current_view=current_view
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
