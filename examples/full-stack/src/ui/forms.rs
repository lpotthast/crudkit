//! The app's create, edit, and read views: CrudKit's form atoms in the app's markup.
//!
//! The forms lay fields out in sections of its own instead of rendering the configured layout with
//! `CrudFormLayout`: every group and run of loose fields becomes a section, every tab a titled
//! section, each holding a responsive grid of its fields.

use crate::ui::inputs::FieldControl;
use crate::ui::{icon, is_read_only, use_noun};
use crudkit_leptos::prelude::*;
use leptos::prelude::*;

/// A group of form fields, titled for tabs. Tabs show as consecutive sections.
struct Section<F> {
    title: Option<String>,
    fields: Vec<(F, FieldOptions)>,
}

/// Returns the fields of a layout in order, flattening nested groups.
fn fields_of<F: Clone>(elements: &[Elem<F>]) -> Vec<(F, FieldOptions)> {
    elements
        .iter()
        .flat_map(|elem| match elem {
            Elem::Field((field, options)) => vec![(field.clone(), options.clone())],
            Elem::Enclosing(Enclosing::None(group) | Enclosing::Card(group)) => {
                fields_of(&group.children)
            }
            Elem::Enclosing(Enclosing::Tabs(tabs)) => tabs
                .iter()
                .flat_map(|tab| fields_of(&tab.group.children))
                .collect(),
            Elem::Separator => Vec::new(),
        })
        .collect()
}

/// Splits a layout into sections: one per top-level group, one per tab, and one per run of loose fields.
fn sections_of<F: Clone>(elements: &[Elem<F>]) -> Vec<Section<F>> {
    let mut sections = Vec::new();
    let mut loose = Vec::new();
    for elem in elements {
        let enclosing = match elem {
            Elem::Field((field, options)) => {
                loose.push((field.clone(), options.clone()));
                continue;
            }
            Elem::Separator => continue,
            Elem::Enclosing(enclosing) => enclosing,
        };
        if !loose.is_empty() {
            sections.push(Section {
                title: None,
                fields: std::mem::take(&mut loose),
            });
        }
        match enclosing {
            Enclosing::None(group) | Enclosing::Card(group) => {
                sections.push(Section {
                    title: None,
                    fields: fields_of(&group.children),
                });
            }
            Enclosing::Tabs(tabs) => sections.extend(tabs.iter().map(|tab| Section {
                title: Some(tab.label.name.clone()),
                fields: fields_of(&tab.group.children),
            })),
        }
    }
    if !loose.is_empty() {
        sections.push(Section {
            title: None,
            fields: loose,
        });
    }
    sections
}

/// Renders `elements` as sections.
fn form_sections<F>(elements: &[Elem<F>]) -> impl IntoView + use<F>
where
    F: TypeErasedField + IntoDynField<Dyn = F>,
{
    sections_of(elements)
        .into_iter()
        .map(|Section { title, fields }| {
            let cells = fields
                .into_iter()
                .map(|(field, options)| {
                    view! {
                        <CrudField field options>
                            <Cell />
                        </CrudField>
                    }
                })
                .collect_view();
            view! {
                <section class="ui-form-section">
                    {title.map(|title| view! { <h3 class="ui-form-section-title">{title}</h3> })}
                    <div class="ui-grid">{cells}</div>
                </section>
            }
        })
        .collect_view()
}

/// The grid cell of the surrounding field. A field with a configured renderer, e.g. a nested
/// instance, spans the whole row.
#[component]
fn Cell() -> impl IntoView {
    let wide = use_crud_field_binding().has_renderer;
    view! {
        <div class="ui-cell" class:ui-cell-wide=wide>
            <FieldControl />
        </div>
    }
}

/// The fields of the surrounding form, or why its entity is not shown.
#[component]
fn FormBody() -> impl IntoView {
    let ctx = use_crud_instance();
    let form = use_crud_form();
    let status = form.status;
    let sections = move || {
        (status.get() == CrudEntityLoadStatus::Ready).then(|| match form.kind {
            CrudEntityViewKind::Create => {
                form_sections(&ctx.create_elements().get_untracked()).into_any()
            }
            CrudEntityViewKind::Update | CrudEntityViewKind::Read => {
                form_sections(&ctx.update_elements().get_untracked()).into_any()
            }
        })
    };
    view! {
        <CrudEntityStatus
            loading=|| {
                view! {
                    <div class="ui-skeleton">
                        <span class="ui-sr-only">"Loading\u{2026}"</span>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                    </div>
                }
            }
            not_found=|| view! { <p class="ui-error">"This record no longer exists."</p> }
            failed=|| view! { <p class="ui-error">"The registry could not be read."</p> }
        />
        {sections}
    }
}

/// The card header shared by all form views: a way back, the view's title, and whether the form holds
/// unsaved changes.
#[component]
fn CardHeader(title: String) -> impl IntoView {
    let noun = use_noun();
    let is_dirty = use_crud_form().is_dirty;
    // A nested instance's views sit below the `h3` naming it in the parent's form.
    let is_nested = use_crud_instance().parent.read_value().is_some();
    let heading = if is_nested {
        view! { <h4>{title}</h4> }.into_any()
    } else {
        view! { <h2>{title}</h2> }.into_any()
    };
    view! {
        <header class="ui-card-header">
            <CrudReturnButton classes="ui-back">
                {icon(icondata::LuArrowLeft)} {format!("All {}", noun.plural)}
            </CrudReturnButton>
            <div class="ui-card-title">
                {heading}
                {move || {
                    is_dirty
                        .get()
                        .then(|| view! { <span class="ui-dirty">"Unsaved changes"</span> })
                }}
            </div>
        </header>
    }
}

/// Reports a successful save in the app's words, instead of CrudKit's default notification.
fn notify_saved() -> Callback<Saved<DynUpdateModel>> {
    let notifier = use_crud_notifier();
    Callback::new(move |_| {
        notifier.notify(CrudNotification::success(
            "Safely kept",
            "Your changes are recorded in the registry.",
        ));
    })
}

#[component]
pub fn CreateView() -> impl IntoView {
    let noun = use_noun();
    view! {
        <CrudCreateForm
            on_saved=notify_saved()
            notifications=CrudSaveNotifications::Failures
            follow_up=CrudSaveFollowUp::Return
            classes="ui-card"
        >
            <CardHeader title=format!("New {}", noun.singular) />
            <div class="ui-card-body">
                <FormBody />
            </div>
            <footer class="ui-card-footer">
                <span></span>
                <div class="ui-button-row">
                    <CrudReturnButton classes="ui-button">"Cancel"</CrudReturnButton>
                    <CrudSaveButton classes=[
                        "ui-button",
                        "ui-button-primary",
                    ]>{format!("Create {}", noun.singular)}</CrudSaveButton>
                </div>
            </footer>
        </CrudCreateForm>
    }
}

#[component]
pub fn EditView(id: SerializableId) -> impl IntoView {
    let noun = use_noun();
    view! {
        <CrudEditForm
            id
            on_saved=notify_saved()
            notifications=CrudSaveNotifications::Failures
            classes="ui-card"
        >
            <CardHeader title=format!("Edit {}", noun.singular) />
            <div class="ui-card-body">
                <FormBody />
            </div>
            <footer class="ui-card-footer">
                <CrudDeleteButton classes=[
                    "ui-button",
                    "ui-button-ghost-danger",
                ]>
                    {icon(icondata::LuTrash2)} {format!("Delete {}", noun.singular)}
                </CrudDeleteButton>
                <CrudSaveButton classes=[
                    "ui-button",
                    "ui-button-primary",
                ]>"Save changes"</CrudSaveButton>
            </footer>
        </CrudEditForm>
    }
}

#[component]
pub fn ReadView(id: SerializableId) -> impl IntoView {
    let noun = use_noun();
    view! {
        <CrudDetails id classes="ui-card">
            <CardHeader title=format!("About this {}", noun.singular) />
            <div class="ui-card-body">
                <FormBody />
            </div>
            {(!is_read_only())
                .then(|| {
                    view! {
                        <footer class="ui-card-footer">
                            <span></span>
                            <CrudEditButton classes=[
                                "ui-button",
                                "ui-button-primary",
                            ]>
                                {icon(icondata::LuPencil)} {format!("Edit {}", noun.singular)}
                            </CrudEditButton>
                        </footer>
                    }
                })}
        </CrudDetails>
    }
}
