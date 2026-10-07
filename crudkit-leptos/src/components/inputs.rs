//! Default input components of field renderers, built on Leptonic's field atoms.

use crate::atoms::icon::CrudIcon;
use crate::hooks::field::use_crud_field;
use crate::hooks::inputs::{
    CrudNumber, CrudTextCodec, CrudValueFormatter, UseCrudNumberInputReturn,
    UseCrudTextInputReturn, UseCrudToggleInputReturn, use_crud_number_input, use_crud_text_input,
    use_crud_toggle_input,
};
use crudkit_web::field::FieldMode;
use crudkit_web::prelude::*;
use leptonic::atoms::prelude::{
    Checkbox, FieldError, Input, Label, NumberField, NumberFieldDecrementButton, NumberFieldGroup,
    NumberFieldIncrementButton, TextArea, TextField,
};
use leptonic::hooks::ValidationBehavior;
use leptonic::utils::number_formatter::NumberFormatOptions;
use leptos::prelude::*;
use std::sync::Arc;

/// Text shown for absent values.
const EMPTY: &str = "\u{2014}";

/// Renders the current value as text when the field is only displayed, e.g. in a table.
fn display_text<F: TypeErasedField>(format: CrudValueFormatter) -> AnyView {
    let value = use_crud_field::<F>().value;
    view! {
        <span class="crudkit-value">
            {move || {
                let text = format(&value.get());
                if text.is_empty() { EMPTY.to_owned() } else { text }
            }}
        </span>
    }
    .into_any()
}

/// Renders a text input for the field being rendered, or its text in display mode.
///
/// `display` formats the value for display mode and defaults to the codec's format.
#[must_use]
pub fn crud_text_input<F: TypeErasedField>(
    codec: CrudTextCodec,
    multiline: bool,
    display: Option<CrudValueFormatter>,
) -> AnyView {
    if use_crud_field::<F>().mode == FieldMode::Display {
        return display_text::<F>(display.unwrap_or_else(|| codec.format.clone()));
    }
    let UseCrudTextInputReturn {
        value,
        set_value,
        props,
    } = use_crud_text_input::<F>(codec);
    let error = props.error;
    let aria_label = props
        .label
        .is_none()
        .then(|| use_crud_field::<F>().field.name().to_string());
    view! {
        <TextField
            value
            set_value
            id=props.id
            is_disabled=props.is_disabled
            is_read_only=props.is_read_only
            is_required=props.is_required
            is_invalid=props.is_invalid
            aria_label=aria_label
            validation_behavior=ValidationBehavior::Aria
            classes="crudkit-input-field"
        >
            {props.label.map(|label| view! { <Label classes="crudkit-label">{label}</Label> })}
            {if multiline {
                view! { <TextArea classes="crudkit-input" /> }.into_any()
            } else {
                view! { <Input classes="crudkit-input" /> }.into_any()
            }}
            <FieldError classes="crudkit-field-error">{move || error.get()}</FieldError>
        </TextField>
    }
    .into_any()
}

/// Renders a number input of type `T` for the field being rendered, or its number in display
/// mode.
#[must_use]
pub fn crud_number_input<F: TypeErasedField, T: CrudNumber + std::fmt::Display>() -> AnyView {
    if use_crud_field::<F>().mode == FieldMode::Display {
        return display_text::<F>(Arc::new(|value| {
            T::from_value(value)
                .map(|it| it.to_string())
                .unwrap_or_default()
        }));
    }
    let UseCrudNumberInputReturn::<T> {
        value,
        set_value,
        props,
    } = use_crud_number_input::<F, T>();
    let error = props.error;
    let aria_label = props
        .label
        .is_none()
        .then(|| use_crud_field::<F>().field.name().to_string());
    view! {
        <NumberField
            value
            set_value
            format_options=NumberFormatOptions {
                use_grouping: false,
                ..NumberFormatOptions::default()
            }
            id=props.id
            is_disabled=props.is_disabled
            is_read_only=props.is_read_only
            is_required=props.is_required
            is_invalid=props.is_invalid
            aria_label=aria_label
            validation_behavior=ValidationBehavior::Aria
            classes="crudkit-input-field"
        >
            {props.label.map(|label| view! { <Label classes="crudkit-label">{label}</Label> })}
            <NumberFieldGroup classes="crudkit-number-group">
                <NumberFieldDecrementButton classes="crudkit-number-step">
                    "\u{2212}"
                </NumberFieldDecrementButton>
                <Input classes="crudkit-input" />
                <NumberFieldIncrementButton classes="crudkit-number-step">
                    "+"
                </NumberFieldIncrementButton>
            </NumberFieldGroup>
            <FieldError classes="crudkit-field-error">{move || error.get()}</FieldError>
        </NumberField>
    }
    .into_any()
}

/// Renders a checkbox for the `bool` field being rendered, or an icon in display mode.
#[must_use]
pub fn crud_toggle_input<F: TypeErasedField>() -> AnyView {
    let field = use_crud_field::<F>();
    if field.mode == FieldMode::Display {
        let value = field.value;
        return view! {
            <span class="crudkit-value">
                {move || match value.get().as_bool() {
                    Some(true) => {
                        view! { <CrudIcon icon=icondata::BsCheckLg classes="crudkit-icon" /> }
                            .into_any()
                    }
                    Some(false) => {
                        view! { <CrudIcon icon=icondata::BsXLg classes="crudkit-icon" /> }
                            .into_any()
                    }
                    None => EMPTY.into_any(),
                }}
            </span>
        }
        .into_any();
    }
    let UseCrudToggleInputReturn {
        is_selected,
        set_selected,
        props,
    } = use_crud_toggle_input::<F>();
    let label = props
        .label
        .unwrap_or_else(|| field.field.name().to_string());
    view! {
        <Checkbox
            is_selected
            set_selected
            id=props.id
            is_disabled=props.is_disabled
            is_read_only=props.is_read_only
            classes="crudkit-checkbox"
        >
            <span class="crudkit-checkbox-box" aria-hidden="true"></span>
            <span class="crudkit-checkbox-label">{label}</span>
        </Checkbox>
    }
    .into_any()
}

/// Renders the `bool` "has validation errors" field being rendered as a warning or check icon.
#[must_use]
pub fn crud_validation_status<F: TypeErasedField>() -> AnyView {
    let value = use_crud_field::<F>().value;
    view! {
        <span class="crudkit-validation-status">
            {move || match value.get().as_bool() {
                Some(true) => {
                    view! {
                        <span data-invalid="true">
                            <CrudIcon
                                icon=icondata::BsExclamationTriangleFill
                                classes="crudkit-icon"
                            />
                        </span>
                    }
                        .into_any()
                }
                Some(false) => {
                    view! {
                        <span>
                            <CrudIcon icon=icondata::BsCheck classes="crudkit-icon" />
                        </span>
                    }
                        .into_any()
                }
                None => EMPTY.into_any(),
            }}
        </span>
    }
    .into_any()
}
