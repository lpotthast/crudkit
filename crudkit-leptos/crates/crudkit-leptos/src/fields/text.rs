use crate::fields::render_label;
use crudkit_core::Value;
use crudkit_web::{FieldMode, FieldOptions};
use leptonic::components::prelude::*;
use leptonic::prelude::TiptapContent;
use leptos::prelude::*;
use std::sync::Arc;

#[component]
pub fn CrudTextField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<String>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! { <div>{move || value.get()}</div> }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TiptapEditor
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    value=value
                    disabled=true
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <TiptapEditor
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    value=value
                    set_value=move |new| {
                        value_changed.run(
                            Ok(Value::String(
                                match new {
                                    TiptapContent::Html(content) => content,
                                    TiptapContent::Json(content) => content,
                                }
                            ))
                        )
                    }
                    disabled=field_options.disabled
                />
            </div>
        }
        .into_any(),
    }
}
