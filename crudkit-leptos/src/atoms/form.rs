//! Forms creating, editing, and showing one entity.

use crate::atoms::content::content_or_default;
use crate::atoms::status::{StatusRegion, status_content};
use crate::config::{CrudEntityViewKind, FieldRendererRegistry};
use crate::hooks::entity::CrudEntityHandle;
use crate::hooks::form::{
    CrudEntityLoadStatus, CrudFormHandle, CrudFormState, CrudSaveFollowUp, CrudSaveNotifications,
    UseCrudCreateFormInput, UseCrudCreateFormReturn, UseCrudEditFormInput, UseCrudEditFormReturn,
    UseCrudReadReturn, expect_crud_form, use_crud_create_form, use_crud_edit_form, use_crud_read,
};
use crate::hooks::instance::{use_crud_instance, use_crud_navigation};
use crate::hooks::texts::use_crud_texts;
use crudkit_core::Saved;
use crudkit_core::id::SerializableId;
use crudkit_web::field::FieldMode;
use crudkit_web::http::RequestError;
use crudkit_web::layout::Elem;
use crudkit_web::prelude::*;
use leptonic::atoms::prelude::{Button, Form, FormContext as LeptonicFormContext};
use leptonic::hooks::{ButtonType, FormValidationContext, ValidationBehavior};
use leptonic::utils::classes::{Classes, MergeStrategy};
use leptonic::utils::data_attributes::flag;
use leptonic::utils::styles::Styles;
use leptos::context::Provider;
use leptos::ev::{KeyboardEvent, SubmitEvent};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys;
use std::any::TypeId;
use std::collections::HashMap;

/// The form surrounding a field atom, typed by its fields: create forms hold create-model fields,
/// edit forms and details update-model fields.
pub(crate) struct FormContext<F: TypeErasedField> {
    pub state: CrudFormState<F>,
    /// The mode of fields that do not set their own.
    pub mode: FieldMode,
    /// The layout configured for the form.
    pub elements: Signal<Vec<Elem<F>>>,
    /// Renderers configured for the form's fields.
    pub renderers: StoredValue<FieldRendererRegistry<F>>,
}

// Derived, it would require `F: Copy`.
#[allow(clippy::expl_impl_clone_on_copy)]
impl<F: TypeErasedField> Clone for FormContext<F> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<F: TypeErasedField> Copy for FormContext<F> {}

/// The `<form>` creating an entity of the surrounding instance: a Leptonic `Form` holding a draft
/// that starts from the create model's defaults, with the parent reference filled in for nested
/// instances. Fields, [`CrudFormLayout`](crate::atoms::CrudFormLayout), and
/// [`CrudSaveButton`] inside act on the draft. Submitting the form saves it.
///
/// HTML forms cannot nest: inside another form, e.g. in a field showing a nested instance, the
/// atom renders a `<div role="group">` instead, which saves on Enter in one of its inputs.
///
/// While the draft has unsaved changes, leaving the view asks for confirmation.
///
/// Data attributes: `data-status` (always `ready`), `data-dirty`, `data-saving`.
///
/// Default classes: `leptonic-Form crudkit-CreateForm`.
#[component]
pub fn CrudCreateForm(
    /// Called after the entity was created, while the form is mounted.
    #[prop(into, optional)]
    on_saved: Option<Callback<Saved<DynUpdateModel>>>,
    /// Called when creating the entity failed, while the form is mounted.
    #[prop(into, optional)]
    on_save_failed: Option<Callback<RequestError>>,
    /// Which outcomes of saving notify the user. Defaults to all.
    #[prop(optional)]
    notifications: CrudSaveNotifications,
    /// What saving does when the saving atom does not say, e.g. on submit. Defaults to opening the
    /// created entity in the edit view.
    #[prop(optional)]
    follow_up: Option<CrudSaveFollowUp>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: Children,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-CreateForm")
        .merge(classes, MergeStrategy::UnionConditions);
    let ctx = use_crud_instance();
    let UseCrudCreateFormReturn {
        form: state,
        save,
        is_saving,
        can_save,
        violations,
    } = use_crud_create_form(UseCrudCreateFormInput {
        on_saved,
        on_save_failed,
        notifications,
    });
    let fields = FormContext {
        state,
        mode: FieldMode::Editable,
        elements: ctx.create_elements(),
        renderers: StoredValue::new(ctx.create_field_renderers()),
    };
    let form = CrudFormHandle {
        kind: CrudEntityViewKind::Create,
        status: Signal::stored(CrudEntityLoadStatus::Ready),
        is_dirty: state.is_dirty,
        has_errors: state.has_errors,
        is_saving,
        can_save,
        save: Some(save),
        violations,
        follow_up: follow_up.unwrap_or(CrudSaveFollowUp::Edit),
        draft: Signal::stored(None),
        field_type: TypeId::of::<DynCreateField>(),
        submits: !is_inside_form_element(),
    };
    view! {
        <Provider value=fields>
            <Provider value=Some(form)>
                <FormRoot form classes styles>
                    {children()}
                </FormRoot>
            </Provider>
        </Provider>
    }
}

/// The `<form>` editing the entity `id` of the surrounding instance: a Leptonic `Form` holding a
/// draft of the loaded entity. Fields, [`CrudFormLayout`](crate::atoms::CrudFormLayout),
/// [`CrudSaveButton`], and [`CrudDeleteButton`](crate::atoms::CrudDeleteButton) inside act on it.
/// Submitting the form saves it.
///
/// HTML forms cannot nest: inside another form, e.g. in a field showing a nested instance, the
/// atom renders a `<div role="group">` instead, which saves on Enter in one of its inputs.
///
/// While the draft has unsaved changes, leaving the view asks for confirmation.
///
/// Data attributes: `data-status` (`loading`, `ready`, `not-found`, `failed`), `data-dirty`,
/// `data-saving`.
///
/// Default classes: `leptonic-Form crudkit-EditForm`.
#[component]
pub fn CrudEditForm(
    /// The edited entity.
    #[prop(into)]
    id: Signal<SerializableId>,
    /// Called after the entity was saved, while the form is mounted.
    #[prop(into, optional)]
    on_saved: Option<Callback<Saved<DynUpdateModel>>>,
    /// Called when saving the entity failed, while the form is mounted.
    #[prop(into, optional)]
    on_save_failed: Option<Callback<RequestError>>,
    /// Which outcomes of saving notify the user. Defaults to all.
    #[prop(optional)]
    notifications: CrudSaveNotifications,
    /// What saving does when the saving atom does not say, e.g. on submit. Defaults to staying.
    #[prop(optional)]
    follow_up: Option<CrudSaveFollowUp>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: Children,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-EditForm")
        .merge(classes, MergeStrategy::UnionConditions);
    let ctx = use_crud_instance();
    let UseCrudEditFormReturn {
        status,
        form: state,
        save,
        is_saving,
        can_save,
        delete,
        can_delete,
        violations,
        ..
    } = use_crud_edit_form(UseCrudEditFormInput {
        id,
        on_saved,
        on_save_failed,
        notifications,
    });
    let fields = FormContext {
        state,
        mode: FieldMode::Editable,
        elements: ctx.update_elements(),
        renderers: StoredValue::new(ctx.update_field_renderers()),
    };
    let form = CrudFormHandle {
        kind: CrudEntityViewKind::Update,
        status,
        is_dirty: state.is_dirty,
        has_errors: state.has_errors,
        is_saving,
        can_save,
        save: Some(save),
        violations,
        follow_up: follow_up.unwrap_or(CrudSaveFollowUp::Stay),
        draft: state.draft,
        field_type: TypeId::of::<DynUpdateField>(),
        submits: !is_inside_form_element(),
    };
    let entity = CrudEntityHandle {
        // Unknown until the entity is shown: it may not exist.
        id: Signal::derive(move || (status.get() == CrudEntityLoadStatus::Ready).then(|| id.get())),
        delete,
        can_delete,
    };
    view! {
        <Provider value=fields>
            <Provider value=Some(form)>
                <Provider value=Some(entity)>
                    <FormRoot form classes styles>
                        {children()}
                    </FormRoot>
                </Provider>
            </Provider>
        </Provider>
    }
}

/// A `<div>` showing the entity `id` of the surrounding instance without editing it. Fields and
/// [`CrudFormLayout`](crate::atoms::CrudFormLayout) inside show its values, read-only.
/// [`CrudEditButton`](crate::atoms::CrudEditButton) and
/// [`CrudDeleteButton`](crate::atoms::CrudDeleteButton) inside act on it.
///
/// Data attributes: `data-status` (`loading`, `ready`, `not-found`, `failed`).
///
/// Default class: `crudkit-Details`.
#[component]
pub fn CrudDetails(
    /// The shown entity.
    #[prop(into)]
    id: Signal<SerializableId>,
    /// How fields show their values: as read-only inputs (the default) or as plain text.
    #[prop(optional)]
    mode: Option<FieldMode>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: Children,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-Details")
        .merge(classes, MergeStrategy::UnionConditions);
    let ctx = use_crud_instance();
    let UseCrudReadReturn {
        status,
        form: state,
        delete,
        can_delete,
        ..
    } = use_crud_read(id);
    let fields = FormContext {
        state,
        mode: mode.unwrap_or(FieldMode::Readable),
        elements: ctx.update_elements(),
        renderers: StoredValue::new(ctx.update_field_renderers()),
    };
    let form = CrudFormHandle {
        kind: CrudEntityViewKind::Read,
        status,
        is_dirty: Signal::stored(false),
        has_errors: Signal::stored(false),
        is_saving: Signal::stored(false),
        can_save: Signal::stored(false),
        save: None,
        violations: Signal::stored(None),
        follow_up: CrudSaveFollowUp::Stay,
        draft: state.draft,
        field_type: TypeId::of::<DynUpdateField>(),
        submits: false,
    };
    let entity = CrudEntityHandle {
        // Unknown until the entity is shown: it may not exist.
        id: Signal::derive(move || (status.get() == CrudEntityLoadStatus::Ready).then(|| id.get())),
        delete,
        can_delete,
    };
    view! {
        <Provider value=fields>
            <Provider value=Some(form)>
                <Provider value=Some(entity)>
                    <div class=classes style=styles data-status=move || data_status(&status.read())>
                        {children()}
                    </div>
                </Provider>
            </Provider>
        </Provider>
    }
}

/// Marks the descendants of a form element, which cannot hold another `<form>`.
#[derive(Debug, Clone, Copy)]
struct InsideFormElement;

/// Returns whether the caller is rendered inside the `<form>` of a form atom, also of an outer
/// instance, e.g. by a field showing a nested instance.
fn is_inside_form_element() -> bool {
    use_context::<InsideFormElement>().is_some()
}

/// The element of a create or edit form: a Leptonic `Form` whose fields validate as the user
/// edits. HTML forms cannot nest, so a form inside another one is a `<div>` instead, which
/// provides the same contexts to its fields and saves on Enter like a form submits.
#[component]
fn FormRoot(
    form: CrudFormHandle,
    classes: Classes,
    styles: Styles,
    children: Children,
) -> impl IntoView {
    let status = form.status;
    let data_status = move || data_status(&status.read());
    let children = view! { <Provider value=InsideFormElement>{children()}</Provider> };
    if form.submits {
        view! {
            <Form
                validation_behavior=ValidationBehavior::Aria
                classes
                styles
                on:submit=move |event| save_on_submit(&event, form)
                attr:data-status=data_status
                attr:data-dirty=flag(form.is_dirty)
                attr:data-saving=flag(form.is_saving)
            >
                {children}
            </Form>
        }
        .into_any()
    } else {
        let validation = FormValidationContext {
            errors: Signal::stored(HashMap::new()),
        };
        let behavior = LeptonicFormContext {
            validation_behavior: ValidationBehavior::Aria,
        };
        view! {
            <div
                role="group"
                class=classes
                style=styles
                on:keydown=move |event| save_on_enter(&event, form)
                data-status=data_status
                data-dirty=flag(form.is_dirty)
                data-saving=flag(form.is_saving)
            >
                <Provider value=behavior>
                    <Provider value=validation>{children}</Provider>
                </Provider>
            </div>
        }
        .into_any()
    }
}

/// Saves `form` instead of letting the browser submit it.
fn save_on_submit(event: &SubmitEvent, form: CrudFormHandle) {
    event.prevent_default();
    save_with_default_follow_up(form);
}

/// Saves `form` on Enter in one of its single-line inputs, as browsers submit a form then. Keeps
/// the form element around it from submitting itself.
fn save_on_enter(event: &KeyboardEvent, form: CrudFormHandle) {
    let in_text_input = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        .is_some_and(|element| {
            element.tag_name().eq_ignore_ascii_case("input")
                && !matches!(
                    element.get_attribute("type").as_deref(),
                    Some("checkbox" | "radio" | "button" | "submit" | "reset")
                )
        });
    if event.key() != "Enter" || !in_text_input || event.default_prevented() {
        return;
    }
    event.prevent_default();
    event.stop_propagation();
    save_with_default_follow_up(form);
}

/// Saves `form` with its own follow-up, if it can be saved now.
fn save_with_default_follow_up(form: CrudFormHandle) {
    if let Some(save) = form.save
        && form.can_save.get_untracked()
    {
        save.run(form.follow_up);
    }
}

/// The `data-status` value of `status`.
fn data_status(status: &CrudEntityLoadStatus) -> &'static str {
    match status {
        CrudEntityLoadStatus::Loading => "loading",
        CrudEntityLoadStatus::Ready => "ready",
        CrudEntityLoadStatus::NotFound => "not-found",
        CrudEntityLoadStatus::Failed(_) => "failed",
    }
}

/// Explains why the surrounding form shows no entity: a `<div>` that is a polite live region
/// (`role="status"`), stating that the entity loads, does not exist, or failed to load, and empty
/// while the form shows the entity.
///
/// Default content: the matching prop, else [`CrudUiTexts::loading`], [`CrudUiTexts::not_found`],
/// or [`CrudUiTexts::data_unavailable`] with the reason. The form's
/// [`status`](crate::hooks::CrudFormHandle::status) tells the reason too.
///
/// Data attributes: `data-status` (`loading`, `not-found`, `failed`, or `ready`).
///
/// Default class: `crudkit-EntityStatus`.
///
/// [`CrudUiTexts::loading`]: crate::config::CrudUiTexts::loading
/// [`CrudUiTexts::not_found`]: crate::config::CrudUiTexts::not_found
/// [`CrudUiTexts::data_unavailable`]: crate::config::CrudUiTexts::data_unavailable
#[component]
pub fn CrudEntityStatus(
    /// Shown while the entity loads.
    #[prop(into, optional)]
    loading: Option<ViewFn>,
    /// Shown when the entity does not exist.
    #[prop(into, optional)]
    not_found: Option<ViewFn>,
    /// Shown when loading the entity failed.
    #[prop(into, optional)]
    failed: Option<ViewFn>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-EntityStatus")
        .merge(classes, MergeStrategy::UnionConditions);
    let status = expect_crud_form("CrudEntityStatus").status;
    let texts = use_crud_texts();
    let data_status = Signal::derive(move || status.with(data_status));
    let content = move || {
        let texts = texts.read();
        match status.get() {
            CrudEntityLoadStatus::Ready => ().into_any(),
            CrudEntityLoadStatus::Loading => {
                status_content(loading.as_ref(), texts.loading.to_string())
            }
            CrudEntityLoadStatus::NotFound => {
                status_content(not_found.as_ref(), texts.not_found.to_string())
            }
            CrudEntityLoadStatus::Failed(error) => status_content(
                failed.as_ref(),
                format!("{}: {error}", texts.data_unavailable),
            ),
        }
    };
    view! {
        <StatusRegion data_status classes styles>
            {content.clone()}
        </StatusRegion>
    }
}

/// Saves the surrounding form, then follows `follow_up`. Pending while the form saves, and disabled
/// while it cannot be saved, e.g. while it holds input that could not be read.
///
/// Without `follow_up`, it is the form's submit button: it saves with the form's follow-up, and
/// pressing Enter in a field of the form saves too.
///
/// Default content: [`CrudUiTexts::save_and_back`] for `follow_up` `Return`,
/// [`CrudUiTexts::save_and_new`] for `CreateAnother`, else [`CrudUiTexts::save`].
///
/// Data attributes: those of Leptonic's `Button`, e.g. `data-pending`.
///
/// Default classes: `leptonic-Button crudkit-SaveButton`.
///
/// [`CrudUiTexts::save`]: crate::config::CrudUiTexts::save
/// [`CrudUiTexts::save_and_back`]: crate::config::CrudUiTexts::save_and_back
/// [`CrudUiTexts::save_and_new`]: crate::config::CrudUiTexts::save_and_new
///
/// # Panics
///
/// Panics outside of a create or edit form, e.g. inside [`CrudDetails`], which do not save.
#[component]
pub fn CrudSaveButton(
    /// What happens after saving. Defaults to the form's `follow_up`.
    #[prop(optional)]
    follow_up: Option<CrudSaveFollowUp>,
    #[prop(into, optional)] is_disabled: Signal<bool>,
    /// Labels the button when its content doesn't, e.g. an icon.
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-SaveButton")
        .merge(classes, MergeStrategy::UnionConditions);
    let form = expect_crud_form("CrudSaveButton");
    let Some(save) = form.save else {
        panic!("`CrudSaveButton` must be rendered inside a create or edit form, not in details");
    };
    let (can_save, is_saving) = (form.can_save, form.is_saving);
    // Without a follow-up of its own, the button submits the form, which saves it with the form's
    // follow-up. Browsers then also submit the form, and save it, on Enter in one of its fields. A
    // form without a `<form>` element, nested in another, saves on press instead.
    let submits = follow_up.is_none() && form.submits;
    let button_type = if submits {
        ButtonType::Submit
    } else {
        ButtonType::Button
    };
    let on_press = move |_| {
        if !submits {
            save.run(follow_up.unwrap_or(form.follow_up));
        }
    };
    let texts = use_crud_texts();
    let text = move || {
        let texts = texts.read();
        match follow_up {
            Some(CrudSaveFollowUp::Return) => texts.save_and_back.to_string(),
            Some(CrudSaveFollowUp::CreateAnother) => texts.save_and_new.to_string(),
            _ => texts.save.to_string(),
        }
    };
    view! {
        <Button
            on_press
            button_type
            is_disabled=Signal::derive(move || {
                (!can_save.get() && !is_saving.get()) || is_disabled.get()
            })
            is_pending=is_saving
            aria_label
            classes
            styles
        >
            {content_or_default(children, text)}
        </Button>
    }
}

/// Returns from the current view, e.g. from an edit view to the list. Asks for confirmation while
/// a form of the view has unsaved changes.
///
/// Default content: [`CrudUiTexts::back`].
///
/// Data attributes: those of Leptonic's `Button`.
///
/// Default classes: `leptonic-Button crudkit-ReturnButton`.
///
/// [`CrudUiTexts::back`]: crate::config::CrudUiTexts::back
#[component]
pub fn CrudReturnButton(
    #[prop(into, optional)] is_disabled: Signal<bool>,
    /// Labels the button when its content doesn't, e.g. an icon.
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-ReturnButton")
        .merge(classes, MergeStrategy::UnionConditions);
    let navigation = use_crud_navigation();
    let texts = use_crud_texts();
    let text = move || texts.read().back.to_string();
    view! {
        <Button
            on_press=move |_| navigation.return_from_current()
            is_disabled
            aria_label
            classes
            styles
        >
            {content_or_default(children, text)}
        </Button>
    }
}
