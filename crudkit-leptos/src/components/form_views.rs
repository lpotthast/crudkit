//! The built-in create, edit, and read views.

use crate::components::action_buttons::CrudEntityActionButtons;
use crate::components::button::CrudButton;
use crate::components::form_layout::CrudFormLayout;
use crate::components::with_classes;
use crate::config::{
    CreateElements, CrudActionsPlacement, CrudBuiltinViewControls, FieldRendererRegistry,
    UpdateElements,
};
use crate::config::{CrudActionIntent, CrudEntityViewKind};
use crate::hooks::form::{
    CrudCreateActions, CrudEntityStatus, CrudSaveFollowUp, UseCrudCreateFormInput,
    UseCrudCreateFormReturn, UseCrudEditFormInput, UseCrudEditFormReturn, UseCrudReadReturn,
    use_crud_create_form, use_crud_edit_form, use_crud_read,
};
use crate::hooks::instance::use_crud_instance;
use crate::hooks::texts::use_crud_texts;
use crate::instance::CrudNavigation;
use crate::instance::{CrudInstanceContext, with_view_overrides};
use crudkit_core::Saved;
use crudkit_core::id::SerializableId;
use crudkit_web::field::FieldMode;
use crudkit_web::http::RequestError;
use crudkit_web::prelude::*;
use leptonic::utils::classes::Classes;
use leptos::prelude::*;

/// A part of the create controls that applications can place with [`CrudActionsOutlet`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrudActionSlot {
    /// The save buttons.
    CreatePrimary,
    /// The return button.
    CreateNavigation,
    /// Save and return buttons together.
    CreateToolbar,
}

/// The create form of the surrounding instance.
///
/// Uses the instance's create elements, or reports that none are configured. Given `controls` or
/// `navigation` also govern the form's save follow-ups and the save controls it publishes.
#[component]
pub fn CrudCreateView(
    /// Defaults to the instance's configuration.
    #[prop(into, optional)]
    create_elements: Option<Signal<CreateElements>>,
    /// Defaults to the instance's configuration.
    #[prop(into, optional)]
    field_renderer_registry: Option<Signal<FieldRendererRegistry<DynCreateField>>>,
    /// Defaults to the instance's configuration.
    #[prop(into, optional)]
    controls: Option<Signal<CrudBuiltinViewControls>>,
    /// Defaults to the navigation of the current view.
    #[prop(optional)]
    navigation: Option<CrudNavigation>,
    /// Called when the entity is successfully created.
    #[prop(into, optional)]
    on_entity_created: Option<Callback<Saved<DynUpdateModel>>>,
    /// Called when entity creation fails, in addition to the error notification.
    #[prop(into, optional)]
    on_entity_creation_failed: Option<Callback<RequestError>>,
    /// Additional classes of the root element.
    #[prop(into, optional)]
    classes: Classes,
) -> impl IntoView {
    with_view_overrides(navigation, controls, move || {
        let ctx = use_crud_instance();
        let create_elements = create_elements.unwrap_or_else(|| ctx.create_elements());
        let field_renderer_registry =
            field_renderer_registry.unwrap_or_else(|| Signal::stored(ctx.create_field_renderers()));
        let texts = use_crud_texts();
        let UseCrudCreateFormReturn { form, actions, .. } =
            use_crud_create_form(UseCrudCreateFormInput {
                on_saved: on_entity_created,
                on_save_failed: on_entity_creation_failed,
                quiet: false,
            });
        let controls = actions.controls;

        view! {
            <div class=with_classes("crudkit-form-view", classes) data-view="create">
                {move || {
                    (controls.get().create_actions_placement == CrudActionsPlacement::Inline)
                        .then(|| view! { <CreateToolbar actions /> })
                }}
                {move || match create_elements.get() {
                    CreateElements::None => {
                        view! { <p class="crudkit-form-empty">{texts.no_fields.to_string()}</p> }
                            .into_any()
                    }
                    CreateElements::Custom(elements) => {
                        view! {
                            <CrudFormLayout
                                field_renderer_registry
                                elements
                                form
                                mode=FieldMode::Editable
                            />
                        }
                            .into_any()
                    }
                }}
            </div>
        }
    })
}

/// Renders the create controls of `context`'s mounted create view in `action_slot`.
#[component]
pub fn CrudActionsOutlet(
    /// The instance whose create controls are rendered, e.g. captured through
    /// `on_context_created`. Renders nothing while `None` or while the instance shows no create
    /// view.
    #[prop(into)]
    context: Signal<Option<CrudInstanceContext>>,
    /// The part of the create controls to render.
    action_slot: CrudActionSlot,
) -> impl IntoView {
    move || match context.get().and_then(|ctx| ctx.create_actions()) {
        Some(actions) => match action_slot {
            CrudActionSlot::CreatePrimary => view! { <CreatePrimaryActions actions /> }.into_any(),
            CrudActionSlot::CreateNavigation => {
                view! { <CreateNavigationActions actions /> }.into_any()
            }
            CrudActionSlot::CreateToolbar => view! { <CreateToolbar actions /> }.into_any(),
        },
        None => ().into_any(),
    }
}

#[component]
fn CreateToolbar(actions: CrudCreateActions) -> impl IntoView {
    view! {
        <div class="crudkit-toolbar" role="toolbar">
            <div class="crudkit-toolbar-start">
                <CreatePrimaryActions actions />
            </div>
            <div class="crudkit-toolbar-end">
                <CreateNavigationActions actions />
            </div>
        </div>
    }
}

#[component]
fn CreatePrimaryActions(actions: CrudCreateActions) -> impl IntoView {
    let CrudCreateActions {
        controls,
        can_save,
        save,
        ..
    } = actions;
    let save_disabled = Signal::derive(move || !can_save.get());
    view! {
        <SaveButtons
            controls
            is_disabled=save_disabled
            save
            primary=CrudSaveFollowUp::Default
            show_save_and_new=Signal::derive(move || controls.get().show_save_and_new)
        />
    }
}

#[component]
fn CreateNavigationActions(actions: CrudCreateActions) -> impl IntoView {
    let CrudCreateActions {
        controls,
        navigation,
        ..
    } = actions;
    view! { <ReturnButton controls navigation /> }
}

/// Save, save-and-return, and save-and-new buttons, as configured by `controls`.
#[component]
fn SaveButtons(
    controls: Signal<CrudBuiltinViewControls>,
    #[prop(into)] is_disabled: Signal<bool>,
    save: Callback<CrudSaveFollowUp>,
    primary: CrudSaveFollowUp,
    #[prop(into)] show_save_and_new: Signal<bool>,
) -> impl IntoView {
    let texts = use_crud_texts();
    let (save_label, back_label, new_label) = (
        StoredValue::new(texts.save.to_string()),
        StoredValue::new(texts.save_and_back.to_string()),
        StoredValue::new(texts.save_and_new.to_string()),
    );
    view! {
        {move || {
            controls
                .get()
                .show_save
                .then(|| {
                    view! {
                        <CrudButton
                            intent=CrudActionIntent::Primary
                            is_disabled
                            on_press=move |_| save.run(primary)
                        >
                            {save_label.get_value()}
                        </CrudButton>
                    }
                })
        }}
        {move || {
            controls
                .get()
                .show_save_and_back
                .then(|| {
                    view! {
                        <CrudButton
                            intent=CrudActionIntent::Primary
                            is_disabled
                            on_press=move |_| save.run(CrudSaveFollowUp::Return)
                        >
                            {back_label.get_value()}
                        </CrudButton>
                    }
                })
        }}
        {move || {
            show_save_and_new
                .get()
                .then(|| {
                    view! {
                        <CrudButton
                            intent=CrudActionIntent::Primary
                            is_disabled
                            on_press=move |_| save.run(CrudSaveFollowUp::CreateAnother)
                        >
                            {new_label.get_value()}
                        </CrudButton>
                    }
                })
        }}
    }
}

#[component]
fn ReturnButton(
    controls: Signal<CrudBuiltinViewControls>,
    navigation: CrudNavigation,
) -> impl IntoView {
    let label = use_crud_texts().back.to_string();
    move || {
        controls.get().show_return.then(|| {
            let label = label.clone();
            view! { <CrudButton on_press=move |_| navigation.return_from_current()>{label}</CrudButton> }
        })
    }
}

#[component]
fn DeleteButton(
    controls: Signal<CrudBuiltinViewControls>,
    #[prop(into)] is_disabled: Signal<bool>,
    delete: Callback<()>,
) -> impl IntoView {
    let label = use_crud_texts().delete.to_string();
    move || {
        controls.get().show_delete.then(|| {
            let label = label.clone();
            view! {
                <CrudButton
                    intent=CrudActionIntent::Danger
                    is_disabled
                    on_press=move |_| delete.run(())
                >
                    {label}
                </CrudButton>
            }
        })
    }
}

/// Explains why an entity is not shown.
#[component]
pub(crate) fn CrudNoData(status: CrudEntityStatus) -> impl IntoView {
    let texts = use_crud_texts();
    let message = match status {
        CrudEntityStatus::Loading | CrudEntityStatus::Ready => return ().into_any(),
        CrudEntityStatus::NotFound => texts.not_found.to_string(),
        CrudEntityStatus::Failed(reason) => format!("{}: {reason}", texts.data_unavailable),
    };
    view! {
        <div class="crudkit-form-status" role="alert">
            {message}
        </div>
    }
    .into_any()
}

/// The edit form of the entity `id` of the surrounding instance.
///
/// Given `controls` or `navigation` also govern the form's dirty guard, save follow-ups, and
/// deletion.
#[component]
pub fn CrudEditView(
    /// The ID of the edited entity.
    #[prop(into)]
    id: Signal<SerializableId>,
    /// Defaults to the instance's configuration.
    #[prop(into, optional)]
    elements: Option<Signal<UpdateElements>>,
    /// Defaults to the instance's configuration.
    #[prop(into, optional)]
    field_renderer_registry: Option<Signal<FieldRendererRegistry<DynUpdateField>>>,
    /// Defaults to the instance's configuration.
    #[prop(into, optional)]
    controls: Option<Signal<CrudBuiltinViewControls>>,
    /// Defaults to the navigation of the current view.
    #[prop(optional)]
    navigation: Option<CrudNavigation>,
    /// Called when the entity is successfully updated.
    #[prop(into, optional)]
    on_entity_updated: Option<Callback<Saved<DynUpdateModel>>>,
    /// Called when updating the entity fails, in addition to the error notification.
    #[prop(into, optional)]
    on_entity_update_failed: Option<Callback<RequestError>>,
    /// Additional classes of the root element.
    #[prop(into, optional)]
    classes: Classes,
) -> impl IntoView {
    with_view_overrides(navigation, controls, move || {
        let ctx = use_crud_instance();
        let elements = elements.unwrap_or_else(|| ctx.update_elements());
        let field_renderer_registry =
            field_renderer_registry.unwrap_or_else(|| Signal::stored(ctx.update_field_renderers()));
        let controls = Signal::derive(move || ctx.builtin_view_controls());
        let navigation = ctx.navigation;
        let UseCrudEditFormReturn {
            entity,
            status,
            form,
            save,
            can_save,
            delete,
            can_delete,
            ..
        } = use_crud_edit_form(UseCrudEditFormInput {
            on_saved: on_entity_updated,
            on_save_failed: on_entity_update_failed,
            ..UseCrudEditFormInput::new(id)
        });

        view! {
            <div
                class=with_classes("crudkit-form-view", classes)
                data-view="edit"
                data-dirty=move || form.is_dirty.get().then_some("true")
            >
                <div class="crudkit-toolbar" role="toolbar">
                    <div class="crudkit-toolbar-start">
                        {move || {
                            entity
                                .read()
                                .loaded()
                                .is_some()
                                .then(|| {
                                    view! {
                                        <SaveButtons
                                            controls
                                            is_disabled=Signal::derive(move || !can_save.get())
                                            save
                                            primary=CrudSaveFollowUp::Stay
                                            show_save_and_new=Signal::derive(move || {
                                                controls.get().show_save_and_new
                                            })
                                        />
                                        <DeleteButton
                                            controls
                                            is_disabled=Signal::derive(move || !can_delete.get())
                                            delete
                                        />
                                        <CrudEntityActionButtons
                                            input=form.draft
                                            required_state=CrudEntityViewKind::Update
                                        />
                                    }
                                })
                        }}
                    </div>
                    <div class="crudkit-toolbar-end">
                        <ReturnButton controls navigation />
                    </div>
                </div>
                {move || match status.get() {
                    CrudEntityStatus::Ready => {
                        view! {
                            <CrudFormLayout
                                field_renderer_registry
                                elements
                                form
                                mode=FieldMode::Editable
                            />
                        }
                            .into_any()
                    }
                    status => view! { <CrudNoData status /> }.into_any(),
                }}
            </div>
        }
    })
}

/// The read-only presentation of the entity `id` of the surrounding instance.
///
/// Uses the update model, like the edit view. Only the table view uses the read model. Given
/// `controls` or `navigation` also govern deletion.
#[component]
pub fn CrudReadView(
    /// The ID of the shown entity.
    #[prop(into)]
    id: Signal<SerializableId>,
    /// Defaults to the instance's configuration.
    #[prop(into, optional)]
    elements: Option<Signal<UpdateElements>>,
    /// Defaults to the instance's configuration.
    #[prop(into, optional)]
    field_renderer_registry: Option<Signal<FieldRendererRegistry<DynUpdateField>>>,
    /// Defaults to the instance's configuration.
    #[prop(into, optional)]
    controls: Option<Signal<CrudBuiltinViewControls>>,
    /// Defaults to the navigation of the current view.
    #[prop(optional)]
    navigation: Option<CrudNavigation>,
    /// Additional classes of the root element.
    #[prop(into, optional)]
    classes: Classes,
) -> impl IntoView {
    with_view_overrides(navigation, controls, move || {
        let ctx = use_crud_instance();
        let elements = elements.unwrap_or_else(|| ctx.update_elements());
        let field_renderer_registry =
            field_renderer_registry.unwrap_or_else(|| Signal::stored(ctx.update_field_renderers()));
        let controls = Signal::derive(move || ctx.builtin_view_controls());
        let navigation = ctx.navigation;
        let UseCrudReadReturn {
            entity,
            status,
            form,
            delete,
            can_delete,
        } = use_crud_read(id);
        let maybe_entity = Signal::derive(move || entity.get().into_loaded());

        view! {
            <div class=with_classes("crudkit-form-view", classes) data-view="read">
                <div class="crudkit-toolbar" role="toolbar">
                    <div class="crudkit-toolbar-start">
                        {move || {
                            entity
                                .read()
                                .loaded()
                                .is_some()
                                .then(|| {
                                    view! {
                                        <DeleteButton
                                            controls
                                            is_disabled=Signal::derive(move || !can_delete.get())
                                            delete
                                        />
                                        <CrudEntityActionButtons
                                            input=maybe_entity
                                            required_state=CrudEntityViewKind::Read
                                        />
                                    }
                                })
                        }}
                    </div>
                    <div class="crudkit-toolbar-end">
                        <ReturnButton controls navigation />
                    </div>
                </div>
                {move || match status.get() {
                    CrudEntityStatus::Ready => {
                        view! {
                            <CrudFormLayout
                                field_renderer_registry
                                elements
                                form
                                mode=FieldMode::Readable
                            />
                        }
                            .into_any()
                    }
                    status => view! { <CrudNoData status /> }.into_any(),
                }}
            </div>
        }
    })
}
