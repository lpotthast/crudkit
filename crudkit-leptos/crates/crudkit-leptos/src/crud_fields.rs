use crudkit_web::{
    prelude::CustomFields, CrudDataTrait, CrudSimpleView, Elem, Enclosing, FieldMode,
};
use leptonic::prelude::*;
use leptos::*;

use crate::crud_field_leptos::CrudField;

// pub enum Msg<T: CrudDataTrait> {
//     ValueChanged((T::Field, Result<Value, String>)),
//     TabSelected(Label),
// }
//
//
// fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
//     match msg {
//         Msg::ValueChanged((field_type, value)) => {
//             ctx.props().value_changed.emit((field_type, value));
//             false
//         }
//         Msg::TabSelected(label) => {
//             ctx.props().on_tab_selection.emit(label);
//             false
//         }
//     }
// }

#[component]
pub fn CrudFields<T>(
    cx: Scope,
    // children: ChildrenRenderer<Item>,
    custom_fields: Signal<CustomFields<T, leptos::View>>,
    api_base_url: Signal<String>,
    #[prop(into)] elements: MaybeSignal<Vec<Elem<T>>>,
    #[prop(into)] entity: Signal<T>,
    mode: FieldMode,
    current_view: CrudSimpleView,
    // value_changed: Callback<(T::Field, Result<Value, String>)>,
    // active_tab: Option<Label>,
    // on_tab_selection: Callback<Label>,
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
                                    //children={ctx.props().children.clone()}
                                    custom_fields=custom_fields
                                    api_base_url=api_base_url
                                    elements=group.children.clone()
                                    entity=entity
                                    mode=mode.clone()
                                    current_view=current_view.clone()
                                    //value_changed={ctx.props().value_changed.clone()}
                                    //active_tab={ctx.props().active_tab.clone()}
                                    //on_tab_selection={ctx.link().callback(|label| Msg::TabSelected(label))}
                                />
                            }.into_view(cx),
                            Enclosing::Tabs(tabs) => view! {cx,
                                <Tabs
                                    //active_tab={ctx.props().active_tab.clone()}
                                    //on_tab_selection={ctx.link().callback(|label| Msg::TabSelected(label))}
                                >
                                    { tabs.into_iter().map(|tab| {
                                        view! {cx,
                                            <Tab name=tab.id label=view! {cx, { tab.label.name.clone() } }>
                                                <CrudFields
                                                    //children={ctx.props().children.clone()}
                                                    custom_fields=custom_fields
                                                    api_base_url=api_base_url
                                                    elements=tab.group.children.clone()
                                                    entity=entity
                                                    mode=mode.clone()
                                                    current_view=current_view.clone()
                                                    //value_changed={ctx.props().value_changed.clone()}
                                                    //active_tab={ctx.props().active_tab.clone()}
                                                    //on_tab_selection={ctx.link().callback(|label| Msg::TabSelected(label))}
                                                />
                                            </Tab>
                                        }
                                    }).collect_view(cx) }
                                </Tabs>
                            }.into_view(cx),
                            Enclosing::Card(group) => view! {cx,
                                <div class={"crud-card"}>
                                    <CrudFields
                                        //children={ctx.props().children.clone()}
                                        custom_fields=custom_fields
                                        api_base_url=api_base_url
                                        elements={group.children.clone()}
                                        entity=entity
                                        mode=mode.clone()
                                        current_view=current_view.clone()
                                        //value_changed={ctx.props().value_changed.clone()}
                                        //active_tab={ctx.props().active_tab.clone()}
                                        //on_tab_selection={ctx.link().callback(|label| Msg::TabSelected(label))}
                                    />
                                </div>
                            }.into_view(cx),
                        }
                    }
                    Elem::Field((field, field_options)) => {
                        view! {cx,
                            // <CrudField
                            //     // children={ctx.props().children.clone()}
                            //     custom_fields={ctx.props().custom_fields.clone()}
                            //     api_base_url={ctx.props().api_base_url.clone()}
                            //     current_view={ctx.props().current_view.clone()}
                            //     // field_type={field_type.clone()}
                            //     // field_options={field_options.clone()}
                            //     // field_mode={ctx.props().mode}
                            //     // entity={ctx.props().entity.clone()}
                            //     // value_changed={ctx.link().callback(Msg::ValueChanged)}
                            // />
                            {move || {
                                let e = entity.get();
                                view!{cx, <CrudField
                                //children={ctx.props().children.clone()} // TODO: make this work
                                custom_fields=custom_fields
                                api_base_url=api_base_url
                                current_view=current_view
                                field=field.clone()
                                field_options=field_options.clone()
                                entity=e
                                field_mode=mode.clone()
                                value_changed=|_, _| {}
                            />
                            }}}
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
