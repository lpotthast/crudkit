//! Marauder's create, edit, and read views: CrudKit's form hooks in Marauder markup.

use crate::marauder::inputs::mrd_input;
use crate::marauder::{icon, use_noun};
use crudkit_leptos::prelude::*;
use leptonic::atoms::prelude::Button;
use leptos::prelude::*;

/// A titled group of form fields. Marauder shows tabs as consecutive sections.
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

/// Renders the fields of `form`: registered renderers where configured, Marauder inputs otherwise.
fn form_sections<F: TypeErasedField>(
    form: CrudFormState<F>,
    elements: &[Elem<F>],
    renderers: &FieldRendererRegistry<F>,
    mode: FieldMode,
) -> impl IntoView + use<F> {
    sections_of(elements)
        .into_iter()
        .map(|Section { title, fields }| {
            let cells = fields
                .into_iter()
                .map(|(field, options)| {
                    let renderer = renderers.get(&field).cloned();
                    let wide = renderer.is_some();
                    view! {
                        <div class="mrd-cell" class:mrd-cell-wide=wide>
                            <CrudFormField form field options mode>
                                {
                                    let renderer = renderer.clone();
                                    move || match renderer.clone() {
                                        Some(renderer) => renderer.render(use_crud_field::<F>()),
                                        None => mrd_input::<F>(),
                                    }
                                }
                            </CrudFormField>
                        </div>
                    }
                })
                .collect_view();
            view! {
                <section class="mrd-form-section">
                    {title.map(|title| view! { <h3 class="mrd-form-section-title">{title}</h3> })}
                    <div class="mrd-grid">{cells}</div>
                </section>
            }
        })
        .collect_view()
}

/// Explains why an entity is not shown yet.
fn entity_message(status: CrudEntityStatus) -> AnyView {
    match status {
        CrudEntityStatus::Loading | CrudEntityStatus::Ready => view! {
            <div class="mrd-skeleton" aria-label="Loading">
                <span></span>
                <span></span>
                <span></span>
            </div>
        }
        .into_any(),
        CrudEntityStatus::NotFound => {
            view! { <p class="mrd-error">"This record no longer exists."</p> }.into_any()
        }
        CrudEntityStatus::Failed(error) => {
            view! { <p class="mrd-error">{error.to_string()}</p> }.into_any()
        }
    }
}

fn notify_saved() -> Callback<Saved<DynUpdateModel>> {
    let notifier = use_crud_notifier();
    Callback::new(move |_| {
        notifier.notify(CrudNotification::success(
            "Mischief managed",
            "Your changes are recorded in the registry.",
        ))
    })
}

/// The card header shared by all form views: a way back and the view's title.
fn card_header(
    navigation: CrudNavigation,
    title: String,
    is_dirty: Option<Signal<bool>>,
) -> impl IntoView {
    let noun = use_noun();
    view! {
        <header class="mrd-card-header">
            <Button classes="mrd-back" on_press=move |_| navigation.return_from_current()>
                {icon(icondata::LuArrowLeft)}
                {format!("All {}", noun.plural)}
            </Button>
            <div class="mrd-card-title">
                <h2>{title}</h2>
                {is_dirty
                    .map(|is_dirty| move || {
                        is_dirty
                            .get()
                            .then(|| view! { <span class="mrd-dirty">"Unsaved changes"</span> })
                    })}
            </div>
        </header>
    }
}

#[component]
pub fn MarauderCreateView(navigation: CrudNavigation) -> impl IntoView {
    let ctx = use_crud_instance();
    let noun = use_noun();
    let UseCrudCreateFormReturn {
        form,
        save,
        can_save,
        ..
    } = use_crud_create_form(UseCrudCreateFormInput {
        on_saved: Some(notify_saved()),
        ..UseCrudCreateFormInput::default()
    });
    let elements = match ctx.create_elements().get_untracked() {
        CreateElements::Custom(elements) => elements,
        CreateElements::None => Vec::new(),
    };
    view! {
        <article class="mrd-card">
            {card_header(navigation, format!("New {}", noun.singular), None)}
            <div class="mrd-card-body">
                {form_sections(form, &elements, &ctx.create_field_renderers(), FieldMode::Editable)}
            </div> <footer class="mrd-card-footer">
                <span></span>
                <div class="mrd-button-row">
                    <Button classes="mrd-button" on_press=move |_| navigation.return_from_current()>
                        "Cancel"
                    </Button>
                    <Button
                        classes=["mrd-button", "mrd-button-primary"]
                        is_disabled=Signal::derive(move || !can_save.get())
                        on_press=move |_| save.run(CrudSaveFollowUp::Return)
                    >
                        {format!("Create {}", noun.singular)}
                    </Button>
                </div>
            </footer>
        </article>
    }
}

#[component]
pub fn MarauderEditView(id: SerializableId, navigation: CrudNavigation) -> impl IntoView {
    let ctx = use_crud_instance();
    let noun = use_noun();
    let UseCrudEditFormReturn {
        status,
        form,
        save,
        can_save,
        delete,
        can_delete,
        ..
    } = use_crud_edit_form(UseCrudEditFormInput {
        on_saved: Some(notify_saved()),
        ..UseCrudEditFormInput::new(id)
    });
    let renderers = ctx.update_field_renderers();
    view! {
        <article class="mrd-card" data-dirty=move || form.is_dirty.get().then_some("true")>
            {card_header(navigation, format!("Edit {}", noun.singular), Some(form.is_dirty))}
            <div class="mrd-card-body">
                {move || match status.get() {
                    CrudEntityStatus::Ready => {
                        form_sections(
                                form,
                                &ctx.update_elements().get_untracked(),
                                &renderers,
                                FieldMode::Editable,
                            )
                            .into_any()
                    }
                    status => entity_message(status),
                }}
            </div>
            <footer class="mrd-card-footer">
                <Button
                    classes=["mrd-button", "mrd-button-ghost-danger"]
                    is_disabled=Signal::derive(move || !can_delete.get())
                    on_press=move |_| delete.run(())
                >
                    {icon(icondata::LuTrash2)}
                    {format!("Delete {}", noun.singular)}
                </Button>
                <Button
                    classes=["mrd-button", "mrd-button-primary"]
                    is_disabled=Signal::derive(move || !can_save.get())
                    on_press=move |_| save.run(CrudSaveFollowUp::Stay)
                >
                    "Save changes"
                </Button>
            </footer>
        </article>
    }
}

#[component]
pub fn MarauderReadView(id: SerializableId, navigation: CrudNavigation) -> impl IntoView {
    let ctx = use_crud_instance();
    let noun = use_noun();
    let UseCrudReadReturn { status, form, .. } = use_crud_read(id);
    let renderers = ctx.update_field_renderers();
    view! {
        <article class="mrd-card">
            {card_header(navigation, format!("About this {}", noun.singular), None)}
            <div class="mrd-card-body">
                {move || match status.get() {
                    CrudEntityStatus::Ready => {
                        form_sections(
                                form,
                                &ctx.update_elements().get_untracked(),
                                &renderers,
                                FieldMode::Readable,
                            )
                            .into_any()
                    }
                    status => entity_message(status),
                }}
            </div>
        </article>
    }
}
