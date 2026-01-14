use crate::shared::fields::render_label;
use crudkit_core::Value;
use crudkit_web::{DateTimeDisplay, FieldMode, FieldOptions};
use leptonic::components::datetime_input::DateTimeInput;
use leptos::prelude::*;
use std::sync::Arc;
use time::PrimitiveDateTime;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;

#[component]
pub fn CrudPrimitiveDateTimeField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<PrimitiveDateTime>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => match field_options.date_time_display {
            DateTimeDisplay::IsoUtc => {
                view! { <div>{move || value.get().format(&Rfc3339).expect("infallible using well-known format")}</div> }.into_any()
            }
            // TODO: Use icu4x formatting using the current users locale!
            DateTimeDisplay::LocalizedLocal => view! {
                <div>
                    {move || {
                        value.get().format(format_description!("[day].[month].[year] [hour]:[minute]")).unwrap()
                    }}
                </div>
            }.into_any(),
        },
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} <div id=id.clone() class="crud-input-field">
                    <DateTimeInput
                        id=id.clone()
                        get=Signal::derive(move || Some(value.get().assume_utc()))
                        set=move |_v| {}
                        disabled=true
                    />
                </div>
            </div>
        }.into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} <div id=id.clone() class="crud-input-field">
                    <DateTimeInput
                        id=id.clone()
                        get=Signal::derive(move || Some(value.get().assume_utc()))
                        set={move |v: Option<time::OffsetDateTime>| {
                            match v {
                                Some(v) => {
                                    value_changed
                                        .run(Ok(Value::PrimitiveDateTime(PrimitiveDateTime::new(v.date(), v.time()))))
                                }
                                None => {}
                            }
                        }}
                        disabled=field_options.disabled
                    />
                </div>
            </div>
        }.into_any(),
    }
}

#[component]
pub fn CrudOptionalPrimitiveDateTimeField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<PrimitiveDateTime>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => match field_options.date_time_display {
            DateTimeDisplay::IsoUtc => {move || match value.get() {
                Some(date_time) => view! { <div>{date_time.format(&Rfc3339).unwrap()}</div> }.into_any(),
                None => view! { <div>""</div> }.into_any(),
            }}.into_any(),
            DateTimeDisplay::LocalizedLocal => {move || match value.get() {
                // TODO: Use icu4x formatting using the current users locale!
                Some(date_time) => view! { <div>{date_time.format(format_description!("[day].[month].[year] [hour]:[minute]")).unwrap()}</div> }.into_any(),
                None => view! { <div>""</div> }.into_any(),
            }}.into_any(),
        },
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} "TODO: DataTime input field"
            // id=id.clone()
            // ty=InputType::Number
            // class="crud-input-field" // TODO: This should not be necessary. We can style the leptonic-input directly.
            // disabled=true
            // get=format!("{value}")
            // set=move |_| {}
            // />
            </div>
        }.into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                "TODO: DataTime input field" {render_label(field_options.label.clone())}
            // <CrudOffsetDatetime
            // id={self.format_id()}
            // value={optional_primitive_date_time.clone().map(|it| it.assume_utc())}
            // onchange={ctx.link().callback(|datetime: Option<time::OffsetDateTime>| Msg::Send(Value::OptionalOffsetDateTime(datetime)))}
            // disabled={options.disabled}
            // />
            </div>
        }.into_any(),
    }
}
