use crate::shared::fields::render_label;
use crudkit_web::{FieldMode, FieldOptions, Value};
use leptonic::components::input::NumberInput;
use leptos::prelude::*;
use std::sync::Arc;

#[component]
pub fn CrudU32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<u32>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get() as f64)
                    set=move |new: f64| { value_changed.run(Ok(Value::U32(new as u32))) }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudU64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<u64>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get() as f64)
                    set=move |new: f64| { value_changed.run(Ok(Value::U64(new as u64))) }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudU128Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<u128>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get() as f64)
                    set=move |new: f64| { value_changed.run(Ok(Value::U128(new as u128))) }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudOptionalU32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<u32>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => {
            move || match value.get() {
                Some(value) => view! { <div>{value}</div> }.into_any(),
                None => view! { <div>"-"</div> }.into_any(),
            }
        }
        .into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                    set=move |new: f64| {
                        value_changed.run(Ok(Value::OptionalU32(Some(new as u32))))
                    }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudOptionalU64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<u64>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => {
            move || match value.get() {
                Some(value) => view! { <div>{value}</div> }.into_any(),
                None => view! { <div>"-"</div> }.into_any(),
            }
        }
        .into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                    set=move |new: f64| {
                        value_changed.run(Ok(Value::OptionalU64(Some(new as u64))))
                    }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudI32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<i32>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get() as f64)
                    set=move |new: f64| { value_changed.run(Ok(Value::I32(new as i32))) }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudOptionalI32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i32>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => {
            move || match value.get() {
                Some(value) => view! { <div>{value}</div> }.into_any(),
                None => view! { <div>"-"</div> }.into_any(),
            }
        }
        .into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                    set=move |new: f64| {
                        value_changed.run(Ok(Value::OptionalI32(Some(new as i32))))
                    }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudI64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<i64>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get() as f64)
                    set=move |new: f64| { value_changed.run(Ok(Value::I64(new as i64))) }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudOptionalI64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i64>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => {
            move || match value.get() {
                Some(value) => view! { <div>{value}</div> }.into_any(),
                None => view! { <div>"-"</div> }.into_any(),
            }
        }
        .into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                    set=move |new: f64| {
                        value_changed.run(Ok(Value::OptionalI64(Some(new as i64))))
                    }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudI128Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<i128>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get() as f64)
                    set=move |new: f64| { value_changed.run(Ok(Value::I128(new as i128))) }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudOptionalI128Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<i128>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => {
            move || match value.get() {
                Some(value) => view! { <div>{value}</div> }.into_any(),
                None => view! { <div>"-"</div> }.into_any(),
            }
        }
        .into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                    set=move |new: f64| {
                        value_changed.run(Ok(Value::OptionalI128(Some(new as i128))))
                    }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudOptionalU128Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<u128>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => {
            move || match value.get() {
                Some(value) => view! { <div>{value}</div> }.into_any(),
                None => view! { <div>"-"</div> }.into_any(),
            }
        }
        .into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get().unwrap_or_default() as f64)
                    set=move |new: f64| {
                        value_changed.run(Ok(Value::OptionalU128(Some(new as u128))))
                    }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudF32Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<f32>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get() as f64)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get() as f64)
                    set=move |new: f64| { value_changed.run(Ok(Value::F32(new as f32))) }
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudF64Field(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<f64>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get())
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <NumberInput
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    disabled=field_options.disabled
                    get=Signal::derive(move || value.get())
                    set=move |new: f64| { value_changed.run(Ok(Value::F64(new))) }
                />
            </div>
        }
        .into_any(),
    }
}
