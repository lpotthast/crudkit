//! Buttons opening and deleting entities of the surrounding instance.

use crate::atoms::content::content_or_default;
use crate::config::CrudUiTexts;
use crate::hooks::entity::use_crud_entity;
use crate::hooks::instance::use_crud_navigation;
use crate::hooks::texts::use_crud_texts;
use crudkit_core::id::SerializableId;
use crudkit_web::view::CrudView;
use leptonic::atoms::prelude::Button;
use leptonic::utils::classes::{Classes, MergeStrategy};
use leptonic::utils::styles::Styles;
use leptos::prelude::*;

/// Opens the create view of the surrounding instance.
///
/// Default content: [`CrudUiTexts::new`].
///
/// Data attributes: those of Leptonic's `Button`.
///
/// Default classes: `leptonic-Button crudkit-CreateButton`.
///
/// [`CrudUiTexts::new`]: crate::config::CrudUiTexts::new
#[component]
pub fn CrudCreateButton(
    #[prop(into, optional)] is_disabled: Signal<bool>,
    /// Labels the button when its content doesn't, e.g. an icon.
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-CreateButton")
        .merge(classes, MergeStrategy::UnionConditions);
    let navigation = use_crud_navigation();
    let texts = use_crud_texts();
    let text = move || texts.read().new.to_string();
    view! {
        <Button
            on_press=move |_| navigation.navigate(CrudView::create())
            is_disabled
            aria_label
            classes
            styles
        >
            {content_or_default(children, text)}
        </Button>
    }
}

/// Opens the read view of the entity of the surrounding table row or form, or of `id`.
///
/// Disabled while the entity is unknown, e.g. while a form loads it.
///
/// Default content: [`CrudUiTexts::view`].
///
/// Data attributes: those of Leptonic's `Button`.
///
/// Default classes: `leptonic-Button crudkit-ReadButton`.
///
/// [`CrudUiTexts::view`]: crate::config::CrudUiTexts::view
#[component]
pub fn CrudReadButton(
    /// The entity to open. Defaults to the entity of the surrounding row or form.
    #[prop(into, optional)]
    id: MaybeProp<SerializableId>,
    #[prop(into, optional)] is_disabled: Signal<bool>,
    /// Labels the button when its content doesn't, e.g. an icon.
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-ReadButton")
        .merge(classes, MergeStrategy::UnionConditions);
    view! {
        <OpenEntityButton
            atom="CrudReadButton"
            id
            open=CrudView::read
            text=|texts: &CrudUiTexts| texts.view.to_string()
            is_disabled
            aria_label
            classes
            styles
            children
        />
    }
}

/// Opens the edit view of the entity of the surrounding table row or form, or of `id`.
///
/// Disabled while the entity is unknown, e.g. while a form loads it.
///
/// Default content: [`CrudUiTexts::edit`].
///
/// Data attributes: those of Leptonic's `Button`.
///
/// Default classes: `leptonic-Button crudkit-EditButton`.
///
/// [`CrudUiTexts::edit`]: crate::config::CrudUiTexts::edit
#[component]
pub fn CrudEditButton(
    /// The entity to open. Defaults to the entity of the surrounding row or form.
    #[prop(into, optional)]
    id: MaybeProp<SerializableId>,
    #[prop(into, optional)] is_disabled: Signal<bool>,
    /// Labels the button when its content doesn't, e.g. an icon.
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-EditButton")
        .merge(classes, MergeStrategy::UnionConditions);
    view! {
        <OpenEntityButton
            atom="CrudEditButton"
            id
            open=CrudView::edit
            text=|texts: &CrudUiTexts| texts.edit.to_string()
            is_disabled
            aria_label
            classes
            styles
            children
        />
    }
}

/// A button opening the view `open` returns for the entity of the surrounding row or form, or of
/// `id`. Shared by the buttons opening an entity (`atom`).
#[component]
fn OpenEntityButton(
    atom: &'static str,
    id: MaybeProp<SerializableId>,
    open: fn(SerializableId) -> CrudView,
    text: fn(&CrudUiTexts) -> String,
    is_disabled: Signal<bool>,
    aria_label: MaybeProp<String>,
    classes: Classes,
    styles: Styles,
    children: Option<Children>,
) -> impl IntoView {
    let navigation = use_crud_navigation();
    let id = entity_id(id, atom);
    let texts = use_crud_texts();
    view! {
        <Button
            on_press=move |_| {
                if let Some(id) = id.get_untracked() {
                    navigation.navigate(open(id));
                }
            }
            is_disabled=Signal::derive(move || id.read().is_none() || is_disabled.get())
            aria_label
            classes
            styles
        >
            {content_or_default(children, move || text(&texts.read()))}
        </Button>
    }
}

/// The id of the entity an atom opens: `id` if given, else the entity of the surrounding row or
/// form.
fn entity_id(id: MaybeProp<SerializableId>, atom: &'static str) -> Signal<Option<SerializableId>> {
    let entity = use_crud_entity();
    if entity.is_none() && id.with_untracked(Option::is_none) {
        tracing::error!(
            atom,
            "the atom is neither given an id nor inside a table row or form"
        );
    }
    Signal::derive(move || {
        id.get()
            .or_else(|| entity.and_then(|entity| entity.id.get()))
    })
}

/// Asks to delete the entity of the surrounding table row or form. Deleting proceeds once the user
/// confirms it in the instance's [`CrudDeleteDialog`](crate::atoms::CrudDeleteDialog).
///
/// Disabled while the entity cannot be deleted, e.g. while a form loads it.
///
/// Default content: [`CrudUiTexts::delete`].
///
/// Data attributes: those of Leptonic's `Button`.
///
/// Default classes: `leptonic-Button crudkit-DeleteButton`.
///
/// [`CrudUiTexts::delete`]: crate::config::CrudUiTexts::delete
#[component]
pub fn CrudDeleteButton(
    #[prop(into, optional)] is_disabled: Signal<bool>,
    /// Labels the button when its content doesn't, e.g. an icon.
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-DeleteButton")
        .merge(classes, MergeStrategy::UnionConditions);
    let Some(entity) = use_crud_entity() else {
        panic!("`CrudDeleteButton` must be rendered inside a table row or an entity form");
    };
    let texts = use_crud_texts();
    let text = move || texts.read().delete.to_string();
    view! {
        <Button
            on_press=move |_| entity.delete.run(())
            is_disabled=Signal::derive(move || !entity.can_delete.get() || is_disabled.get())
            aria_label
            classes
            styles
        >
            {content_or_default(children, text)}
        </Button>
    }
}
