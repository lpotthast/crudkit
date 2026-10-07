//! Rendering of field layouts.

use crate::components::field::form_field;
use crate::config::FieldRendererRegistry;
use crate::hooks::form::CrudFormState;
use crate::hooks::tabs::{UseCrudTabSelectionReturn, use_crud_tab_selection};
use crudkit_web::field::FieldMode;
use crudkit_web::layout::Layout;
use crudkit_web::prelude::*;
use leptonic::atoms::prelude::Separator;
use leptonic::atoms::tabs::{Tab, TabList, TabPanel, Tabs};
use leptonic::hooks::{Key, use_collection};
use leptos::prelude::*;

fn columns(layout: &Layout) -> &'static str {
    match layout {
        Layout::Columns1 => "1",
        Layout::Columns2 => "2",
        Layout::Columns3 => "3",
        Layout::Columns4 => "4",
    }
}

/// Renders a field layout: groups (`div.crudkit-group` with `data-columns`), cards
/// (`section.crudkit-card`), tabs, separators, and fields.
///
/// Tabs keep the instance's tab selection (see [`use_crud_tab_selection`]), so the layout must be
/// rendered inside a CrudKit instance.
#[component]
pub fn CrudFormLayout<F: TypeErasedField>(
    /// Renderer overrides of the fields; other fields use their default renderer.
    field_renderer_registry: Signal<FieldRendererRegistry<F>>,
    /// The layout to render, e.g. [`CrudInstanceConfig::elements`].
    ///
    /// [`CrudInstanceConfig::elements`]: crate::config::CrudInstanceConfig::elements
    #[prop(into)]
    elements: Signal<Vec<Elem<F>>>,
    /// The form holding the values of the laid-out fields.
    form: CrudFormState<F>,
    /// Mode of every rendered field.
    mode: FieldMode,
) -> impl IntoView {
    move || {
        elements
            .get()
            .into_iter()
            .map(|elem| render_elem(elem, field_renderer_registry, form, mode))
            .collect_view()
    }
}

fn render_group<F: TypeErasedField>(
    group: Group<F>,
    class: &'static str,
    field_renderer_registry: Signal<FieldRendererRegistry<F>>,
    form: CrudFormState<F>,
    mode: FieldMode,
) -> AnyView {
    let Group { layout, children } = group;
    view! {
        <div class=class data-columns=columns(&layout)>
            <CrudFormLayout field_renderer_registry elements=children form mode />
        </div>
    }
    .into_any()
}

fn render_elem<F: TypeErasedField>(
    elem: Elem<F>,
    field_renderer_registry: Signal<FieldRendererRegistry<F>>,
    form: CrudFormState<F>,
    mode: FieldMode,
) -> AnyView {
    match elem {
        Elem::Enclosing(Enclosing::None(group)) => {
            render_group(group, "crudkit-group", field_renderer_registry, form, mode)
        }
        Elem::Enclosing(Enclosing::Card(group)) => view! {
            <section class="crudkit-card">
                {render_group(group, "crudkit-group", field_renderer_registry, form, mode)}
            </section>
        }
        .into_any(),
        Elem::Enclosing(Enclosing::Tabs(tabs)) => {
            render_tabs(tabs, field_renderer_registry, form, mode)
        }
        Elem::Field((field, field_options)) => {
            form_field(field_renderer_registry, form, &field, field_options, mode)
        }
        Elem::Separator => view! { <Separator classes="crudkit-separator" /> }.into_any(),
    }
}

fn render_tabs<F: TypeErasedField>(
    tabs: Vec<crudkit_web::layout::Tab<F>>,
    field_renderer_registry: Signal<FieldRendererRegistry<F>>,
    form: CrudFormState<F>,
    mode: FieldMode,
) -> AnyView {
    let UseCrudTabSelectionReturn {
        selected_key,
        set_selected_key,
    } = use_crud_tab_selection(tabs.iter().map(|tab| tab.id.clone()).collect());
    let labels = tabs
        .iter()
        .map(|tab| (tab.id.to_string(), tab.label.name.clone()))
        .collect::<Vec<_>>();
    let collection_labels = labels.clone();
    let collection = use_collection(move |b| {
        for (id, label) in &collection_labels {
            b.item(Key::from(id.clone()), label.clone());
        }
    });
    // Tabs and panels read the context of `Tabs` and `TabList`, so they are created as their
    // children rather than up front.
    view! {
        <Tabs collection selected_key set_selected_key classes="crudkit-tabs">
            <TabList classes="crudkit-tab-list">
                {labels
                    .into_iter()
                    .map(|(id, label)| {
                        view! {
                            <Tab key=Key::from(id) classes="crudkit-tab">
                                {label}
                            </Tab>
                        }
                    })
                    .collect_view()}
            </TabList>
            {tabs
                .into_iter()
                .map(|tab| {
                    let group = tab.group;
                    view! {
                        <TabPanel key=Key::from(tab.id.to_string()) classes="crudkit-tab-panel">
                            {
                                let group = group.clone();
                                move || {
                                    render_group(
                                        group.clone(),
                                        "crudkit-group",
                                        field_renderer_registry,
                                        form,
                                        mode,
                                    )
                                }
                            }
                        </TabPanel>
                    }
                })
                .collect_view()}
        </Tabs>
    }
    .into_any()
}
