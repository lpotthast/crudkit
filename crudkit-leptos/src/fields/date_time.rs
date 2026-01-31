use crate::fields::optional::OptionalInput;
use crate::fields::render_label;
use crudkit_core::Value;
use crudkit_web::{DateTimeDisplay, FieldMode, FieldOptions};
use leptonic::components::datetime_input::DateTimeInput;
use leptos::prelude::*;
use std::sync::Arc;
use time::PrimitiveDateTime;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;

/// Returns a default PrimitiveDateTime (Unix epoch).
fn default_datetime() -> PrimitiveDateTime {
    time::macros::datetime!(1970-01-01 00:00:00)
}

/// PrimitiveDateTime field component that handles both required and optional fields.
/// The signal always holds Option<PrimitiveDateTime> - Some(value) for values, None for null.
#[component]
pub fn CrudPrimitiveDateTimeField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<PrimitiveDateTime>>,
    is_optional: bool,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    let datetime_value = Signal::derive(move || value.get().map(|dt| dt.assume_utc()));

    match field_mode {
        FieldMode::Display => match field_options.date_time_display {
            DateTimeDisplay::IsoUtc => {
                move || match value.get() {
                    Some(dt) => view! { {dt.format(&Rfc3339).expect("infallible using well-known format")} }.into_any(),
                    None => view! { "-" }.into_any(),
                }
            }
            .into_any(),
            DateTimeDisplay::LocalizedLocal => {
                move || match value.get() {
                    // TODO: Use icu4x formatting using the current users locale!
                    Some(dt) => view! { {dt.format(format_description!("[day].[month].[year] [hour]:[minute]")).expect("infallible with valid format")} }.into_any(),
                    None => view! { "-" }.into_any(),
                }
            }
            .into_any(),
        },

        FieldMode::Readable if is_optional => {
            let id_inner = id.clone();
            view! {
                {render_label(field_options.label.clone())}
                <div id=id.clone() class="crud-input-field">
                    <OptionalInput
                        get=value
                        set={move |_: Option<PrimitiveDateTime>| {}}
                        disabled=true
                        default_provider=default_datetime
                        input_renderer={move |_disabled_or_null| view! {
                            <DateTimeInput
                                id=id_inner.clone()
                                get=datetime_value
                                set={move |_v| {}}
                                disabled=true
                            />
                        }}
                    />
                </div>
            }
        }.into_any(),

        FieldMode::Readable => view! {
            {render_label(field_options.label.clone())}
            <div id=id.clone() class="crud-input-field">
                <DateTimeInput
                    id=id.clone()
                    get=datetime_value
                    set=move |_v| {}
                    disabled=true
                />
            </div>
        }.into_any(),

        FieldMode::Editable if is_optional => {
            let disabled = field_options.disabled;
            let id_inner = id.clone();
            view! {
                {render_label(field_options.label.clone())}
                <div id=id.clone() class="crud-input-field">
                    <OptionalInput
                        get=value
                        set={move |opt| {
                            let val = match opt {
                                Some(dt) => Value::PrimitiveDateTime(dt),
                                None => Value::Null,
                            };
                            value_changed.run(Ok(val));
                        }}
                        disabled
                        default_provider=default_datetime
                        input_renderer={move |disabled_or_null| view! {
                            <DateTimeInput
                                id=id_inner.clone()
                                get=datetime_value
                                set={move |v: Option<time::OffsetDateTime>| {
                                    let val = match v {
                                        Some(v) => Value::PrimitiveDateTime(PrimitiveDateTime::new(v.date(), v.time())),
                                        None => Value::Null,
                                    };
                                    value_changed.run(Ok(val));
                                }}
                                disabled=disabled_or_null
                            />
                        }}
                    />
                </div>
            }
        }.into_any(),

        FieldMode::Editable => view! {
            {render_label(field_options.label.clone())}
            <div id=id.clone() class="crud-input-field">
                <DateTimeInput
                    id=id.clone()
                    get=datetime_value
                    set={move |v: Option<time::OffsetDateTime>| {
                        let val = match v {
                            Some(v) => Value::PrimitiveDateTime(PrimitiveDateTime::new(v.date(), v.time())),
                            None => Value::Null,
                        };
                        value_changed.run(Ok(val));
                    }}
                    disabled=field_options.disabled
                />
            </div>
        }.into_any(),
    }
}
