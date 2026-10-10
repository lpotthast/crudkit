//! The configured layout of the surrounding form.

use crate::atoms::content::content_or_default;
use crate::atoms::field::{CrudField, CrudFieldControl};
use crate::atoms::form::FormContext;
use crate::config::CrudEntityViewKind;
use crate::hooks::form::expect_crud_form;
use crate::hooks::tabs::{UseCrudTabSelectionReturn, tab_key, use_crud_tab_selection};
use crudkit_web::layout::{Elem, Enclosing, Group, Layout, Tab as LayoutTab, TabId};
use crudkit_web::prelude::*;
use leptonic::atoms::prelude::Separator;
use leptonic::atoms::tabs::{Tab, TabList, TabPanel, Tabs};
use leptonic::hooks::use_collection;
use leptonic::utils::classes::{Classes, MergeStrategy};
use leptonic::utils::styles::Styles;
use leptos::context::Provider;
use leptos::prelude::*;

/// The layout configured for the surrounding form, rendered from atoms. Renders no element of its
/// own.
///
/// The layout is the instance's [`create_elements`] in a create form and its [`elements`] in an
/// edit form or details. Groups render as [`CrudFormGroup`]s, cards as [`CrudFormCard`]s holding a
/// group, tabs as [`CrudFormTabs`] with a Leptonic `TabList` of [`CrudFormTab`]s and a
/// [`CrudFormTabPanel`] per tab (holding a group, and kept mounted while another tab is shown, so
/// its fields keep their input), separators as [`CrudFormSeparator`]s, and every
/// field as a [`CrudField`] holding `field`. The `*_classes` props add classes to each element of a
/// kind.
///
/// [`create_elements`]: crate::config::CrudInstanceConfig::create_elements
/// [`elements`]: crate::config::CrudInstanceConfig::elements
#[component]
// The props are named after the atoms they style, e.g. `CrudFormTabs` and `CrudFormTab`.
#[allow(clippy::similar_names)]
pub fn CrudFormLayout(
    /// The markup of each field, rendered inside its [`CrudField`]. Defaults to a
    /// [`CrudFieldControl`].
    #[prop(into, optional)]
    field: Option<ViewFn>,
    /// Classes of each [`CrudFormGroup`].
    #[prop(into, optional)]
    group_classes: Classes,
    /// Classes of each [`CrudFormCard`].
    #[prop(into, optional)]
    card_classes: Classes,
    /// Classes of each [`CrudFormTabs`].
    #[prop(into, optional)]
    tabs_classes: Classes,
    /// Classes of each Leptonic `TabList`.
    #[prop(into, optional)]
    tab_list_classes: Classes,
    /// Classes of each [`CrudFormTab`].
    #[prop(into, optional)]
    tab_classes: Classes,
    /// Classes of each [`CrudFormTabPanel`].
    #[prop(into, optional)]
    tab_panel_classes: Classes,
    /// Classes of each [`CrudFormSeparator`].
    #[prop(into, optional)]
    separator_classes: Classes,
) -> impl IntoView {
    let field = field.unwrap_or_else(|| ViewFn::from(|| view! { <CrudFieldControl /> }));
    let form = expect_crud_form("CrudFormLayout");
    let classes = LayoutClasses {
        group: group_classes,
        card: card_classes,
        tabs: tabs_classes,
        tab_list: tab_list_classes,
        tab: tab_classes,
        tab_panel: tab_panel_classes,
        separator: separator_classes,
    };
    let elements = match form.kind {
        CrudEntityViewKind::Create => use_context::<FormContext<DynCreateField>>()
            .map(|form| view! { <FormElements elements=form.elements field /> }.into_any()),
        CrudEntityViewKind::Update | CrudEntityViewKind::Read => {
            use_context::<FormContext<DynUpdateField>>()
                .map(|form| view! { <FormElements elements=form.elements field /> }.into_any())
        }
    };
    view! { <Provider value=classes>{elements}</Provider> }
}

/// The classes a [`CrudFormLayout`] adds to the elements it renders.
#[derive(Debug, Clone)]
struct LayoutClasses {
    group: Classes,
    card: Classes,
    tabs: Classes,
    tab_list: Classes,
    tab: Classes,
    tab_panel: Classes,
    separator: Classes,
}

fn use_layout_classes() -> LayoutClasses {
    expect_context::<LayoutClasses>()
}

/// The layout `elements` of a form, re-rendered when they change.
#[component]
fn FormElements<F: TypeErasedField + IntoDynField<Dyn = F>>(
    elements: Signal<Vec<Elem<F>>>,
    field: ViewFn,
) -> impl IntoView {
    move || view! { <LayoutElements elements=elements.get() field=field.clone() /> }
}

/// A sequence of layout elements.
#[component]
fn LayoutElements<F: TypeErasedField + IntoDynField<Dyn = F>>(
    elements: Vec<Elem<F>>,
    field: ViewFn,
) -> impl IntoView {
    elements
        .into_iter()
        .map(|elem| view! { <LayoutElement elem field=field.clone() /> })
        .collect_view()
}

/// One layout element. Erased into an [`AnyView`], because groups nest elements recursively.
#[component]
fn LayoutElement<F: TypeErasedField + IntoDynField<Dyn = F>>(
    elem: Elem<F>,
    field: ViewFn,
) -> AnyView {
    match elem {
        Elem::Field((bound, options)) => {
            let field = StoredValue::new(field);
            view! {
                <CrudField field=bound options>
                    {move || field.with_value(ViewFn::run)}
                </CrudField>
            }
            .into_any()
        }
        Elem::Enclosing(Enclosing::None(group)) => view! { <LayoutGroup group field /> }.into_any(),
        Elem::Enclosing(Enclosing::Card(group)) => view! {
            <CrudFormCard classes=use_layout_classes().card>
                <LayoutGroup group field />
            </CrudFormCard>
        }
        .into_any(),
        Elem::Enclosing(Enclosing::Tabs(tabs)) => view! { <LayoutTabs tabs field /> }.into_any(),
        Elem::Separator => {
            view! { <CrudFormSeparator classes=use_layout_classes().separator /> }.into_any()
        }
    }
}

/// A group of layout elements, laid out in its columns.
#[component]
fn LayoutGroup<F: TypeErasedField + IntoDynField<Dyn = F>>(
    group: Group<F>,
    field: ViewFn,
) -> impl IntoView {
    let Group { layout, children } = group;
    view! {
        <CrudFormGroup columns=layout classes=use_layout_classes().group>
            <LayoutElements elements=children field />
        </CrudFormGroup>
    }
}

/// Tabs of layout groups.
#[component]
fn LayoutTabs<F: TypeErasedField + IntoDynField<Dyn = F>>(
    tabs: Vec<LayoutTab<F>>,
    field: ViewFn,
) -> impl IntoView {
    let classes = use_layout_classes();
    let labels = tabs
        .iter()
        .map(|tab| CrudTab {
            id: tab.id.clone(),
            label: tab.label.name.clone(),
        })
        .collect::<Vec<_>>();
    let ids = labels.iter().map(|tab| tab.id.clone()).collect::<Vec<_>>();
    let field = StoredValue::new(field);
    let tab_classes = classes.tab.clone();
    let tab_panel_classes = classes.tab_panel.clone();
    // Tabs and panels read the context of `CrudFormTabs` and `TabList`, so they are created as
    // their children rather than up front.
    view! {
        <CrudFormTabs tabs=labels classes=classes.tabs>
            <TabList classes=classes.tab_list>
                {ids
                    .into_iter()
                    .map(|id| view! { <CrudFormTab id classes=tab_classes.clone() /> })
                    .collect_view()}
            </TabList>
            {tabs
                .into_iter()
                .map(|tab| {
                    let group = StoredValue::new(tab.group);
                    view! {
                        <CrudFormTabPanel
                            id=tab.id
                            should_force_mount=true
                            classes=tab_panel_classes.clone()
                        >
                            {move || {
                                view! {
                                    <LayoutGroup group=group.get_value() field=field.get_value() />
                                }
                            }}
                        </CrudFormTabPanel>
                    }
                })
                .collect_view()}
        </CrudFormTabs>
    }
}

/// A `<div>` grouping fields of a form in `columns` columns.
///
/// Data attributes: `data-columns` (`1` to `4`, e.g. for a CSS grid).
///
/// Default class: `crudkit-FormGroup`.
#[component]
pub fn CrudFormGroup(
    /// How many columns the group's fields are laid out in.
    #[prop(optional)]
    columns: Layout,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: Children,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-FormGroup")
        .merge(classes, MergeStrategy::UnionConditions);
    let columns = match columns {
        Layout::Columns1 => "1",
        Layout::Columns2 => "2",
        Layout::Columns3 => "3",
        Layout::Columns4 => "4",
    };
    view! {
        <div class=classes style=styles data-columns=columns>
            {children()}
        </div>
    }
}

/// A `<section>` setting a part of a form apart, e.g. as a card.
///
/// Default class: `crudkit-FormCard`.
#[component]
pub fn CrudFormCard(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: Children,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-FormCard")
        .merge(classes, MergeStrategy::UnionConditions);
    view! {
        <section class=classes style=styles>
            {children()}
        </section>
    }
}

/// A tab of [`CrudFormTabs`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrudTab {
    /// The tab's id, which its [`CrudFormTab`] and [`CrudFormTabPanel`] take.
    pub id: TabId,
    /// The tab's label.
    pub label: String,
}

/// The tabs of the surrounding [`CrudFormTabs`].
#[derive(Debug, Clone)]
struct FormTabs(Vec<CrudTab>);

/// Tabs of a form: a Leptonic `Tabs` whose selection is the instance's, so it survives switching
/// views. Compose it from a Leptonic `TabList` with a [`CrudFormTab`] per tab, and a
/// [`CrudFormTabPanel`] per tab.
///
/// Data attributes: those of Leptonic's `Tabs`.
///
/// Default classes: `leptonic-Tabs crudkit-FormTabs`.
#[component]
pub fn CrudFormTabs(
    /// The tabs, in order.
    tabs: Vec<CrudTab>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: Children,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-FormTabs")
        .merge(classes, MergeStrategy::UnionConditions);
    let UseCrudTabSelectionReturn {
        selected_key,
        set_selected_key,
    } = use_crud_tab_selection(tabs.iter().map(|tab| tab.id.clone()).collect());
    let collection = {
        let tabs = tabs.clone();
        use_collection(move |builder| {
            for tab in &tabs {
                builder.item(tab_key(&tab.id), tab.label.clone());
            }
        })
    };
    view! {
        <Provider value=FormTabs(tabs)>
            <Tabs collection selected_key set_selected_key classes styles>
                {children()}
            </Tabs>
        </Provider>
    }
}

/// The tab `id` of the surrounding [`CrudFormTabs`]: a Leptonic `Tab` inside its `TabList`.
///
/// Default content: the tab's label.
///
/// Data attributes: those of Leptonic's `Tab`.
///
/// Default classes: `leptonic-Tab crudkit-FormTab`.
#[component]
pub fn CrudFormTab(
    /// The tab's id.
    id: TabId,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-FormTab")
        .merge(classes, MergeStrategy::UnionConditions);
    let label = use_context::<FormTabs>()
        .and_then(|FormTabs(tabs)| tabs.into_iter().find(|tab| tab.id == id))
        .map(|tab| tab.label)
        .unwrap_or_default();
    view! {
        <Tab key=tab_key(&id) classes styles>
            {content_or_default(children, label)}
        </Tab>
    }
}

/// The panel of the tab `id` of the surrounding [`CrudFormTabs`]: a Leptonic `TabPanel`, holding
/// the tab's content while the tab is selected, or always with `should_force_mount`.
///
/// Data attributes: those of Leptonic's `TabPanel`.
///
/// Default classes: `leptonic-TabPanel crudkit-FormTabPanel`.
#[component]
pub fn CrudFormTabPanel(
    /// The tab's id.
    id: TabId,
    /// Keeps the panel mounted, hidden and inert, while another tab is selected. Its fields then
    /// keep input that could not be read yet, and its error, when the user switches tabs.
    #[prop(optional)]
    should_force_mount: bool,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: ChildrenFn,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-FormTabPanel")
        .merge(classes, MergeStrategy::UnionConditions);
    view! {
        <TabPanel key=tab_key(&id) should_force_mount classes styles>
            {children()}
        </TabPanel>
    }
}

/// A Leptonic `Separator` between parts of a form.
///
/// Default classes: `leptonic-Separator crudkit-FormSeparator`.
#[component]
pub fn CrudFormSeparator(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-FormSeparator")
        .merge(classes, MergeStrategy::UnionConditions);
    view! { <Separator classes styles /> }
}
