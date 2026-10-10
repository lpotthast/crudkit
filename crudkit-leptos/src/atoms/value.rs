//! A value shown as text.

use crate::hooks::inputs::format_crud_value;
use crate::hooks::texts::use_crud_texts;
use crudkit_core::{Value, ValueKind};
use crudkit_web::field::DateTimeDisplay;
use leptonic::utils::classes::{Classes, MergeStrategy};
use leptonic::utils::styles::Styles;
use leptos::prelude::*;

/// A `<span>` showing `value` as text, formatted by [`format_crud_value`].
///
/// Data attributes: `data-kind` (`text`, `number`, `bool`, `date-time`, `duration`, `uuid`,
/// `json`, `list`, or `other`, e.g. to align numbers) and `data-empty` (an absent value, e.g. to
/// show a dash with CSS).
///
/// Default class: `crudkit-Value`.
#[component]
pub fn CrudValue(
    /// The shown value.
    #[prop(into)]
    value: Signal<Value>,
    /// How date-times without offset are shown.
    #[prop(optional)]
    date_time_display: DateTimeDisplay,
    #[prop(into, optional)] classes: Classes,
    #[prop(into, optional)] styles: Styles,
) -> impl IntoView {
    let classes = Classes::new()
        .add("crudkit-Value")
        .merge(classes, MergeStrategy::UnionConditions);
    let texts = use_crud_texts();
    let is_empty = move || value.with(|value| matches!(value, Value::Null | Value::Void(())));
    let text =
        move || value.with(|value| format_crud_value(value, date_time_display, &texts.read()));
    view! {
        <span
            class=classes
            style=styles
            data-kind=move || value.with(|value| data_kind(value.kind()))
            data-empty=move || is_empty().then_some("true")
        >
            {text}
        </span>
    }
}

/// The `data-kind` value of values of `kind`: a coarse kind for styling, e.g. to align numbers.
/// `None` for absent values.
pub(crate) fn data_kind(kind: ValueKind) -> Option<&'static str> {
    Some(match kind {
        ValueKind::Null | ValueKind::Void => return None,
        ValueKind::Bool => "bool",
        ValueKind::U8
        | ValueKind::U16
        | ValueKind::U32
        | ValueKind::U64
        | ValueKind::U128
        | ValueKind::I8
        | ValueKind::I16
        | ValueKind::I32
        | ValueKind::I64
        | ValueKind::I128
        | ValueKind::F32
        | ValueKind::F64 => "number",
        ValueKind::String => "text",
        ValueKind::Json => "json",
        ValueKind::Uuid => "uuid",
        ValueKind::PrimitiveDateTime | ValueKind::OffsetDateTime => "date-time",
        ValueKind::Duration => "duration",
        ValueKind::Array => "list",
        ValueKind::Other => "other",
    })
}
