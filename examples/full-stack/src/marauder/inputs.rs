//! Marauder's field inputs: Leptonic's field atoms bound to fields through CrudKit's input hooks.

use crudkit_leptos::prelude::*;
use leptonic::atoms::prelude::{
    Description, Input, Label, NumberField, Switch, TextArea, TextField,
};
use leptonic::utils::number_formatter::NumberFormatOptions;
use leptos::prelude::*;

/// Renders an input for the field in context. Use it inside `CrudFormField`.
pub fn mrd_input<F: TypeErasedField>() -> AnyView {
    let kind = use_crud_field::<F>().field.value_kind();
    // Matching exhaustively makes the compiler point out value kinds without an input.
    match CrudInputKind::of(kind) {
        CrudInputKind::Nothing => ().into_any(),
        CrudInputKind::Toggle => toggle::<F>(),
        CrudInputKind::Number(kind) => match kind {
            CrudNumberKind::U8 => number::<F, u8>(),
            CrudNumberKind::U16 => number::<F, u16>(),
            CrudNumberKind::U32 => number::<F, u32>(),
            CrudNumberKind::U64 => number::<F, u64>(),
            CrudNumberKind::U128 => number::<F, u128>(),
            CrudNumberKind::I8 => number::<F, i8>(),
            CrudNumberKind::I16 => number::<F, i16>(),
            CrudNumberKind::I32 => number::<F, i32>(),
            CrudNumberKind::I64 => number::<F, i64>(),
            CrudNumberKind::I128 => number::<F, i128>(),
            CrudNumberKind::F32 => number::<F, f32>(),
            CrudNumberKind::F64 => number::<F, f64>(),
        },
        CrudInputKind::Text { codec, multiline } => text::<F>(codec, multiline),
        CrudInputKind::Custom => {
            view! { <p class="mrd-error">"This field needs a registered renderer."</p> }.into_any()
        }
    }
}

fn label(props: &CrudInputProps) -> impl IntoView + use<> {
    props
        .label
        .clone()
        .map(|label| view! { <Label classes="mrd-label">{label}</Label> })
}

fn error(props: &CrudInputProps) -> impl IntoView + use<> {
    let error = props.error;
    move || {
        error
            .get()
            .map(|error| view! { <Description classes="mrd-field-error">{error}</Description> })
    }
}

fn text<F: TypeErasedField>(codec: CrudTextCodec, multiline: bool) -> AnyView {
    let UseCrudTextInputReturn {
        value,
        set_value,
        props,
    } = use_crud_text_input::<F>(codec);
    view! {
        <TextField
            value
            set_value
            id=props.id.clone()
            is_disabled=props.is_disabled
            is_read_only=props.is_read_only
            is_required=props.is_required
            is_invalid=props.is_invalid
            classes="mrd-field"
        >
            {label(&props)}
            {if multiline {
                view! { <TextArea classes="mrd-input" /> }.into_any()
            } else {
                view! { <Input classes="mrd-input" /> }.into_any()
            }}
            {error(&props)}
        </TextField>
    }
    .into_any()
}

fn number<F: TypeErasedField, T: CrudNumber>() -> AnyView {
    let UseCrudNumberInputReturn::<T> {
        value,
        set_value,
        props,
    } = use_crud_number_input::<F, T>();
    // Marauder's numbers are ids and years, which read wrong with thousands separators.
    let format_options = NumberFormatOptions {
        use_grouping: false,
        ..NumberFormatOptions::default()
    };
    view! {
        <NumberField
            value
            set_value
            format_options
            id=props.id.clone()
            is_disabled=props.is_disabled
            is_read_only=props.is_read_only
            is_required=props.is_required
            classes="mrd-field"
        >
            {label(&props)}
            <Input classes="mrd-input" />
            {error(&props)}
        </NumberField>
    }
    .into_any()
}

fn toggle<F: TypeErasedField>() -> AnyView {
    let UseCrudToggleInputReturn {
        is_selected,
        set_selected,
        props,
    } = use_crud_toggle_input::<F>();
    let label = props.label.clone().unwrap_or_default();
    view! {
        <Switch
            is_selected
            set_selected
            is_disabled=props.is_disabled
            is_read_only=props.is_read_only
            classes="mrd-switch"
        >
            <span class="mrd-switch-track" aria-hidden="true"></span>
            {label}
        </Switch>
    }
    .into_any()
}
