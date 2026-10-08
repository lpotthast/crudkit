//! Fields of the surrounding form, bound to Leptonic's field atoms.

use crate::atoms::content::content_or_default;
use crate::atoms::form::FormContext;
use crate::atoms::value::CrudValue;
use crate::hooks::field::{
    CrudFieldBinding, CrudFieldInputError, CrudFieldState, dom_id, use_crud_field_binding,
};
use crate::hooks::form::{CrudFormState, expect_crud_form};
use crate::hooks::inputs::{
    CrudInputKind, CrudNumber, CrudNumberKind, CrudTextCodec, UseCrudNumberInputReturn,
    UseCrudTextInputReturn, UseCrudToggleInputReturn, use_crud_number_input, use_crud_text_input,
    use_crud_toggle_input,
};
use crate::hooks::texts::use_crud_texts;
use crudkit_core::Value;
use crudkit_web::field::{FieldMode, FieldOptions};
use crudkit_web::layout::{Elem, Enclosing};
use crudkit_web::prelude::*;
use leptonic::atoms::prelude::{
    Checkbox, FieldError, Input, Label, LabelContext, NumberField, Switch, TextArea, TextField,
};
use leptonic::hooks::UseLabelProps;
use leptonic::utils::classes::{Classes, MergeStrategy};
use leptonic::utils::number_formatter::NumberFormatOptions;
use leptonic::utils::styles::Styles;
use leptos::context::Provider;
use leptos::prelude::*;
use std::any::TypeId;

/// The renderer registered for the surrounding field in the instance configuration, if any.
#[derive(Clone)]
struct RegisteredRenderer(Option<ViewFn>);

/// Binds `field` of the surrounding form to its children. Renders no element of its own.
///
/// Inside, field atoms bind to it, e.g. [`CrudTextField`] with its [`CrudFieldLabel`] and
/// [`CrudFieldError`], or [`CrudFieldControl`] for the markup matching the field's kind. The
/// field must belong to the form: a create-model field inside a [`CrudCreateForm`], an
/// update-model field inside a [`CrudEditForm`] or [`CrudDetails`].
///
/// The children are re-created when the form loads another entity. A field the form's model does
/// not have shows a visible configuration error instead: a
/// `<div class="crudkit-UnknownField" role="alert">`.
///
/// [`CrudCreateForm`]: crate::atoms::CrudCreateForm
/// [`CrudEditForm`]: crate::atoms::CrudEditForm
/// [`CrudDetails`]: crate::atoms::CrudDetails
#[component]
pub fn CrudField<T: IntoDynField>(
    /// The bound field, e.g. `Club::Name`.
    field: T,
    /// The field's label and other options. Defaults to the options the form's layout gives the
    /// field, if it contains it.
    #[prop(optional)]
    options: Option<FieldOptions>,
    /// How the field is presented. Defaults to the form's mode: editable in create and edit
    /// forms, read-only in details.
    #[prop(optional)]
    mode: Option<FieldMode>,
    children: ChildrenFn,
) -> impl IntoView {
    let field = field.into_dyn();
    let form = use_form_of::<T::Dyn>(&field);
    let options = options.unwrap_or_else(|| {
        form.elements
            .with_untracked(|elements| configured_options(elements, &field))
            .unwrap_or_default()
    });
    let mode = mode.unwrap_or(form.mode);
    let renderer = form
        .renderers
        .with_value(|renderers| renderers.get(&field).cloned());
    let name = field.name().to_string();
    let field_state = move || form_field_state(form.state, &field, options.clone(), mode);
    (move || {
        let Some(state) = field_state() else {
            // A form still loading its entity has no values yet.
            let is_loading = form
                .state
                .fields
                .read()
                .with_value(std::collections::HashMap::is_empty);
            return (!is_loading)
                .then(|| view! { <UnknownField field=name.clone() /> })
                .into_any();
        };
        let binding = CrudFieldBinding {
            has_renderer: renderer.is_some(),
            ..CrudFieldBinding::from(&state)
        };
        let registered = RegisteredRenderer(renderer.clone().map(|renderer| {
            let state = state.clone();
            // Inside the renderer, `CrudFieldControl` renders the default markup, so renderers
            // can decorate it.
            ViewFn::from(move || {
                let view = renderer.render(state.clone());
                view! { <Provider value=RegisteredRenderer(None)>{view}</Provider> }
            })
        }));
        let children = children.clone();
        view! {
            <Provider value=state>
                <Provider value=binding>
                    <Provider value=registered>{children()}</Provider>
                </Provider>
            </Provider>
        }
        .into_any()
    })
    .into_any()
}

/// A visible configuration error: the form's model has no `field`, e.g. a layout naming a field
/// of another model.
#[component]
fn UnknownField(field: String) -> impl IntoView {
    tracing::error!(
        field,
        "a field is bound that the form's model does not have"
    );
    let texts = use_crud_texts();
    let message = move || (texts.read().unknown_field)(&field);
    view! {
        <div class="crudkit-UnknownField" role="alert">
            {message}
        </div>
    }
}

/// Returns the surrounding form of `field`.
///
/// # Panics
///
/// Panics outside of a form, or when the form's fields are not of type `F`, e.g. for a field of
/// the update model inside a create form.
fn use_form_of<F: TypeErasedField>(field: &F) -> FormContext<F> {
    let form = expect_crud_form("CrudField");
    assert!(
        form.field_type == TypeId::of::<F>(),
        "`CrudField` binds `{}`, a field of another model than its form's",
        field.name()
    );
    expect_context::<FormContext<F>>()
}

/// Returns the options the layout `elements` give `field`, if they contain it.
fn configured_options<F: TypeErasedField>(elements: &[Elem<F>], field: &F) -> Option<FieldOptions> {
    elements.iter().find_map(|elem| match elem {
        Elem::Field((candidate, options)) => (candidate == field).then(|| options.clone()),
        Elem::Enclosing(Enclosing::None(group) | Enclosing::Card(group)) => {
            configured_options(&group.children, field)
        }
        Elem::Enclosing(Enclosing::Tabs(tabs)) => tabs
            .iter()
            .find_map(|tab| configured_options(&tab.group.children, field)),
        Elem::Separator => None,
    })
}

/// Returns the state of `field` in `form`, or `None` if the form has no value for it. Tracks the
/// form's values, so a reloaded entity yields new state.
fn form_field_state<F: TypeErasedField>(
    form: CrudFormState<F>,
    field: &F,
    options: FieldOptions,
    mode: FieldMode,
) -> Option<CrudFieldState<F>> {
    let value = form.field(field)?;
    let target = field.clone();
    Some(CrudFieldState {
        field: field.clone(),
        value,
        mode,
        options,
        fields: form.fields.get_untracked(),
        set: Callback::new(move |result: Result<Value, CrudFieldInputError>| {
            form.set_field(target.clone(), result.map_err(|err| err.to_string()));
        }),
        error: form.field_error(field.clone()),
        is_saving: form.is_saving,
        dom_id: dom_id(),
    })
}

/// The markup for the surrounding field: the [`FieldRenderer`](crate::config::FieldRenderer)
/// registered for it in the instance configuration, else the markup for its input kind. Renders
/// no element of its own. Inside a registered renderer, it renders the markup for the input kind,
/// so a renderer can wrap the default markup.
///
/// The defaults for the input kinds, each replaceable through its prop:
///
/// - `text`: a [`CrudTextField`] with a [`CrudFieldLabel`], a Leptonic `Input` (a `TextArea` for
///   multiline values, e.g. JSON), and a [`CrudFieldError`];
/// - `number`: a [`CrudNumberField`] with a [`CrudFieldLabel`], an `Input`, and a
///   [`CrudFieldError`];
/// - `toggle`: a [`CrudSwitch`];
/// - `display`, for fields in display mode: a [`CrudDisplayField`] with a [`CrudFieldLabel`] and a
///   [`CrudFieldValue`].
///
/// Values without an input (the unit value) render nothing. Lists and custom types have no default
/// markup and show a visible configuration error until a renderer is registered: a
/// `<div class="crudkit-MissingFieldRenderer" role="alert">`.
#[component]
pub fn CrudFieldControl(
    /// Markup for text, JSON, UUID, date-time, and duration values.
    #[prop(into, optional)]
    text: Option<ViewFn>,
    /// Markup for numbers.
    #[prop(into, optional)]
    number: Option<ViewFn>,
    /// Markup for `bool` values.
    #[prop(into, optional)]
    toggle: Option<ViewFn>,
    /// Markup for fields in display mode, whatever their kind.
    #[prop(into, optional)]
    display: Option<ViewFn>,
) -> impl IntoView {
    if let Some(RegisteredRenderer(Some(renderer))) = use_context::<RegisteredRenderer>() {
        return renderer.run();
    }
    let binding = use_crud_field_binding();
    if binding.mode == FieldMode::Display {
        return match display {
            Some(display) => display.run(),
            None => view! {
                <CrudDisplayField>
                    <CrudFieldLabel />
                    <CrudFieldValue />
                </CrudDisplayField>
            }
            .into_any(),
        };
    }
    match CrudInputKind::of(binding.value_kind) {
        CrudInputKind::Nothing => ().into_any(),
        CrudInputKind::Toggle => match toggle {
            Some(toggle) => toggle.run(),
            None => view! { <CrudSwitch /> }.into_any(),
        },
        CrudInputKind::Number(_) => match number {
            Some(number) => number.run(),
            None => view! {
                <CrudNumberField>
                    <CrudFieldLabel />
                    <Input />
                    <CrudFieldError />
                </CrudNumberField>
            }
            .into_any(),
        },
        CrudInputKind::Text { multiline, .. } => match text {
            Some(text) => text.run(),
            None => view! {
                <CrudTextField>
                    <CrudFieldLabel />
                    {if multiline {
                        view! { <TextArea /> }.into_any()
                    } else {
                        view! { <Input /> }.into_any()
                    }}
                    <CrudFieldError />
                </CrudTextField>
            }
            .into_any(),
        },
        CrudInputKind::Custom => view! { <MissingRenderer field=binding.name /> }.into_any(),
    }
}

/// A visible configuration error: `field` has a custom type and no registered renderer.
#[component]
fn MissingRenderer(field: String) -> impl IntoView {
    tracing::error!(field, "a field of a custom type has no registered renderer");
    let texts = use_crud_texts();
    let message = move || (texts.read().no_renderer)(&field);
    view! {
        <div class="crudkit-MissingFieldRenderer" role="alert">
            {message}
        </div>
    }
}

/// Binds the surrounding field to a Leptonic `TextField`, editing its value as text. Compose it
/// from a [`CrudFieldLabel`], a Leptonic `Input` or `TextArea`, and a [`CrudFieldError`].
///
/// Data attributes: those of Leptonic's `TextField`.
///
/// Default classes: `leptonic-TextField crudkit-TextField`.
#[component]
pub fn CrudTextField(
    /// Converts between the field's value and text. Defaults to the codec of the field's kind.
    #[prop(optional)]
    codec: Option<CrudTextCodec>,
    #[prop(into, optional)] placeholder: MaybeProp<String>,
    /// Names the field when it has no [`CrudFieldLabel`].
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: Children,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-TextField")
        .merge(classes, MergeStrategy::UnionConditions);
    let codec =
        codec.unwrap_or_else(
            || match CrudInputKind::of(use_crud_field_binding().value_kind) {
                CrudInputKind::Text { codec, .. } => codec,
                _ => CrudTextCodec::string(),
            },
        );
    let UseCrudTextInputReturn {
        value,
        set_value,
        props,
    } = use_crud_text_input(codec);
    view! {
        <TextField
            value
            set_value
            id=props.id
            is_disabled=props.is_disabled
            is_read_only=props.is_read_only
            is_required=props.is_required
            is_invalid=props.is_invalid
            placeholder
            aria_label
            classes
            styles
        >
            {children()}
        </TextField>
    }
}

/// Binds the surrounding number field to a Leptonic `NumberField` of the field's number type.
/// Compose it from a [`CrudFieldLabel`], a Leptonic `Input`, and a [`CrudFieldError`].
///
/// Numbers are shown without grouping digits by default, because IDs and years are the most
/// common numbers in CRUD forms.
///
/// Data attributes: those of Leptonic's `NumberField`.
///
/// Default classes: `leptonic-NumberField crudkit-NumberField`.
///
/// # Panics
///
/// Panics when bound to a field that does not hold numbers.
#[component]
pub fn CrudNumberField(
    /// How numbers are shown. Defaults to Leptonic's options without grouping digits.
    #[prop(into, optional)]
    format_options: Option<Signal<NumberFormatOptions>>,
    /// Names the field when it has no [`CrudFieldLabel`].
    #[prop(into, optional)]
    aria_label: MaybeProp<String>,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: Children,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-NumberField")
        .merge(classes, MergeStrategy::UnionConditions);
    let format_options = format_options.unwrap_or_else(|| {
        Signal::stored(NumberFormatOptions {
            use_grouping: false,
            ..NumberFormatOptions::default()
        })
    });
    let binding = use_crud_field_binding();
    let CrudInputKind::Number(kind) = CrudInputKind::of(binding.value_kind) else {
        panic!(
            "`CrudNumberField` is bound to `{}`, a field that does not hold numbers",
            binding.name
        );
    };
    // The field's number type is only known at runtime; each arm binds the field as that type.
    macro_rules! number_field {
        ($number:ty) => {{
            let input = use_crud_number_input::<$number>();
            view! {
                <TypedNumberField input format_options aria_label classes styles>
                    {children()}
                </TypedNumberField>
            }
            .into_any()
        }};
    }
    match kind {
        CrudNumberKind::U8 => number_field!(u8),
        CrudNumberKind::U16 => number_field!(u16),
        CrudNumberKind::U32 => number_field!(u32),
        CrudNumberKind::U64 => number_field!(u64),
        CrudNumberKind::U128 => number_field!(u128),
        CrudNumberKind::I8 => number_field!(i8),
        CrudNumberKind::I16 => number_field!(i16),
        CrudNumberKind::I32 => number_field!(i32),
        CrudNumberKind::I64 => number_field!(i64),
        CrudNumberKind::I128 => number_field!(i128),
        CrudNumberKind::F32 => number_field!(f32),
        CrudNumberKind::F64 => number_field!(f64),
    }
}

/// [`CrudNumberField`] for a field holding numbers of type `T`, bound through `input`.
#[component]
fn TypedNumberField<T: CrudNumber>(
    input: UseCrudNumberInputReturn<T>,
    format_options: Signal<NumberFormatOptions>,
    aria_label: MaybeProp<String>,
    classes: Classes,
    styles: Styles,
    children: Children,
) -> impl IntoView {
    let UseCrudNumberInputReturn {
        value,
        set_value,
        props,
    } = input;
    view! {
        <NumberField
            value
            set_value
            format_options
            id=props.id
            is_disabled=props.is_disabled
            is_read_only=props.is_read_only
            is_required=props.is_required
            is_invalid=props.is_invalid
            aria_label
            classes
            styles
        >
            {children()}
        </NumberField>
    }
}

/// Binds the surrounding `bool` field to a Leptonic `Switch`.
///
/// Default content: the field's label.
///
/// Data attributes: those of Leptonic's `Switch`.
///
/// Default classes: `leptonic-Switch crudkit-Switch`.
#[component]
pub fn CrudSwitch(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-Switch")
        .merge(classes, MergeStrategy::UnionConditions);
    let label = use_crud_field_binding().label();
    let UseCrudToggleInputReturn {
        is_selected,
        set_selected,
        props,
    } = use_crud_toggle_input();
    view! {
        <Switch
            is_selected
            set_selected
            id=props.id
            is_disabled=props.is_disabled
            is_read_only=props.is_read_only
            is_invalid=props.is_invalid
            classes
            styles
        >
            {content_or_default(children, label)}
        </Switch>
    }
}

/// Binds the surrounding `bool` field to a Leptonic `Checkbox`.
///
/// Default content: the field's label.
///
/// Data attributes: those of Leptonic's `Checkbox`.
///
/// Default classes: `leptonic-Checkbox crudkit-Checkbox`.
#[component]
pub fn CrudCheckbox(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-Checkbox")
        .merge(classes, MergeStrategy::UnionConditions);
    let label = use_crud_field_binding().label();
    let UseCrudToggleInputReturn {
        is_selected,
        set_selected,
        props,
    } = use_crud_toggle_input();
    view! {
        <Checkbox
            is_selected
            set_selected
            id=props.id
            is_disabled=props.is_disabled
            is_read_only=props.is_read_only
            is_invalid=props.is_invalid
            classes
            styles
        >
            {content_or_default(children, label)}
        </Checkbox>
    }
}

/// Shows the surrounding field without editing it: a `div` grouping its label and value, e.g. for
/// read views. Compose it from a [`CrudFieldLabel`] and a [`CrudFieldValue`]; the label names the
/// group (`role="group"`, `aria-labelledby`).
///
/// Default class: `crudkit-DisplayField`.
#[component]
pub fn CrudDisplayField(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    children: Children,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-DisplayField")
        .merge(classes, MergeStrategy::UnionConditions);
    let binding = use_crud_field_binding();
    let label_id = format!("{}-label", binding.dom_id);
    let label = LabelContext::span(UseLabelProps {
        id: label_id.clone(),
        html_for: None,
    });
    view! {
        <div
            id=binding.dom_id
            role="group"
            aria-labelledby=label_id
            class=classes
            style=styles
        >
            <Provider value=label>{children()}</Provider>
        </div>
    }
}

/// The label of the surrounding field: a Leptonic `Label` inside a field atom, e.g. a
/// [`CrudTextField`] or a [`CrudDisplayField`].
///
/// Default content: the field's label: the one its options give, else the field's name.
///
/// Default classes: `leptonic-Label crudkit-FieldLabel`.
#[component]
pub fn CrudFieldLabel(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-FieldLabel")
        .merge(classes, MergeStrategy::UnionConditions);
    let label = use_crud_field_binding().label();
    view! {
        <Label classes styles>
            {content_or_default(children, label)}
        </Label>
    }
}

/// The error of the surrounding field: a Leptonic `FieldError` inside a field atom, e.g. a
/// [`CrudTextField`], rendered only while the field is invalid.
///
/// Default content: why the field's input could not be read.
///
/// Default classes: `leptonic-FieldError crudkit-FieldError`.
#[component]
pub fn CrudFieldError(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
    #[prop(optional)] children: Option<ChildrenFn>,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-FieldError")
        .merge(classes, MergeStrategy::UnionConditions);
    let error = use_crud_field_binding().error;
    let children = StoredValue::new(children);
    view! {
        <FieldError classes styles>
            {move || {
                children
                    .with_value(|children| match children {
                        Some(children) => children().into_any(),
                        None => error.get().unwrap_or_default().into_any(),
                    })
            }}
        </FieldError>
    }
}

/// The value of the surrounding field as text: a [`CrudValue`], e.g. for fields in display mode.
///
/// Data attributes: those of [`CrudValue`].
///
/// Default classes: `crudkit-Value crudkit-FieldValue`.
#[component]
pub fn CrudFieldValue(
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-FieldValue")
        .merge(classes, MergeStrategy::UnionConditions);
    let binding = use_crud_field_binding();
    let value = binding.value;
    view! {
        <CrudValue
            value=Signal::derive(move || value.get())
            date_time_display=binding.options.date_time_display
            classes
            styles
        />
    }
}
