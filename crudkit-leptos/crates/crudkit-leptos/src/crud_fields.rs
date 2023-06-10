use std::collections::HashMap;

use crudkit_web::{
    prelude::CustomFields, CrudDataTrait, CrudSimpleView, Elem, Enclosing, FieldMode, TabId, Value,
};
use leptonic::prelude::*;
use leptos::*;

use crate::{crud_field_leptos::CrudField, crud_instance_config::DynSelectConfig};

// TODO: Propagate tab selection...

#[component]
pub fn CrudFields<T>(
    cx: Scope,
    // children: ChildrenRenderer<Item>,
    custom_fields: Signal<CustomFields<T, leptos::View>>,
    field_config: Signal<HashMap<T::Field, DynSelectConfig>>,
    api_base_url: Signal<String>,
    #[prop(into)] elements: MaybeSignal<Vec<Elem<T>>>,
    #[prop(into)] entity: Signal<T>,
    mode: FieldMode,
    current_view: CrudSimpleView,
    value_changed: Callback<(T::Field, Result<Value, String>)>,
    // active_tab: Option<Label>,
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
                match elem {
                    Elem::Enclosing(enclosing) => {
                        match enclosing {
                            Enclosing::None(group) => view! {cx,
                                <CrudFields
                                    custom_fields=custom_fields
                                    field_config=field_config
                                    api_base_url=api_base_url
                                    elements=group.children.clone()
                                    entity=entity
                                    mode=mode.clone()
                                    current_view=current_view.clone()
                                    value_changed=value_changed
                                    //active_tab={ctx.props().active_tab.clone()}
                                    on_tab_selection=on_tab_selection
                                />
                            }.into_view(cx),
                            Enclosing::Tabs(tabs) => view! {cx,
                                <Tabs
                                    //active_tab={ctx.props().active_tab.clone()}
                                    //on_tab_selection={ctx.link().callback(|label| Msg::TabSelected(label))}
                                >
                                    { tabs.into_iter().map(|tab| {
                                        let id = tab.id.clone();
                                        view! {cx,
                                            <Tab name=tab.id label=tab.label.name.clone().into_view(cx) on_show=Callback::new(cx, move |()| { on_tab_selection.call(id.clone()) })>
                                                <CrudFields
                                                    custom_fields=custom_fields
                                                    field_config=field_config
                                                    api_base_url=api_base_url
                                                    elements=tab.group.children.clone()
                                                    entity=entity
                                                    mode=mode.clone()
                                                    current_view=current_view.clone()
                                                    value_changed=value_changed
                                                    //active_tab={ctx.props().active_tab.clone()}
                                                    on_tab_selection=on_tab_selection
                                                />
                                            </Tab>
                                        }
                                    }).collect_view(cx) }
                                </Tabs>
                            }.into_view(cx),
                            Enclosing::Card(group) => view! {cx,
                                <div class={"crud-card"}> // TODO: Use leptonic card
                                    <CrudFields
                                        custom_fields=custom_fields
                                        field_config=field_config
                                        api_base_url=api_base_url
                                        elements={group.children.clone()}
                                        entity=entity
                                        mode=mode.clone()
                                        current_view=current_view.clone()
                                        value_changed=value_changed
                                        //active_tab={ctx.props().active_tab.clone()}
                                        on_tab_selection=on_tab_selection
                                    />
                                </div>
                            }.into_view(cx),
                        }
                    }
                    Elem::Field((field, field_options)) => {
                        view!{cx,
                            <CrudField
                                custom_fields=custom_fields
                                field_config=field_config
                                api_base_url=api_base_url
                                current_view=current_view
                                field=field.clone()
                                field_options=field_options.clone()
                                entity=entity
                                field_mode=mode.clone()
                                value_changed=value_changed
                            />
                        }.into_view(cx)
                    },
                    Elem::Separator => view! {cx,
                        <Separator />
                    }
                    .into_view(cx),
                }
            })
            .collect_view(cx)
    }
}
