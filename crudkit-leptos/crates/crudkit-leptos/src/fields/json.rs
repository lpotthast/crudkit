use crate::fields::render_label;
use crudkit_core::Value;
use crudkit_web::{FieldMode, FieldOptions};
use leptonic::components::prelude::*;
use leptonic::prelude::TiptapContent;
use leptos::prelude::*;
use std::borrow::Cow;
use std::sync::Arc;

#[component]
pub fn CrudJsonField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<serde_json::Value>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => {
            view! { <div>{move || serde_json::to_string(&*value.read()).unwrap()}</div> }
        }
        .into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                // TODO: Implement a proper Json editor
                {render_label(field_options.label.clone())}
                <TiptapEditor
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    value=Signal::derive(move || serde_json::to_string(&*value.read()).unwrap())
                    disabled=true
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                // TODO: Implement a proper Json editor
                {render_label(field_options.label.clone())}
                <TiptapEditor
                    attr:id=id.clone()
                    attr:class="crud-input-field"
                    value=Signal::derive(move || serde_json::to_string(&*value.read()).unwrap())
                    set_value=move |new| {
                        value_changed.run(
                            match new {
                                TiptapContent::Html(content) => serde_json::from_str(&content),
                                TiptapContent::Json(content) => serde_json::from_str(&content),
                            }
                                .map(|json_value| Value::Json(json_value))
                                .map_err(|err| Arc::new(err) as Arc<dyn std::error::Error>)
                        );
                    }

                    disabled=field_options.disabled
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudOptionalJsonField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<serde_json::Value>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => view! {
            <div>
                {move || {
                    value
                        .get()
                        .as_ref()
                        .map(|it| Cow::Owned(serde_json::to_string(it).unwrap()))
                        .unwrap_or(Cow::Borrowed(""))
                }}

            </div>
        }.into_any(),
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} "TODO: Implement TipTap editor or Json editor"
            // <CrudTipTapEditor
            // api_base_url={ctx.props().api_base_url.clone()}
            // id={self.format_id()}
            // class={"crud-input-field"}
            // value={value.as_ref().map(|it| it.get_string_representation().to_owned()).unwrap_or_default()}
            // disabled={true}
            // />
            </div>
        }.into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())} "TODO: Implement TipTap editor or Json editor"
            // <CrudTipTapEditor
            // api_base_url={ctx.props().api_base_url.clone()}
            // id={self.format_id()}
            // class={"crud-input-field"}
            // value={value.as_ref().map(|it| it.get_string_representation().to_owned()).unwrap_or_default()}
            // onchange={ctx.link().callback(|input| Msg::Send(Value::Text(input)))}
            // disabled={options.disabled}
            // />
            </div>
        }.into_any(),
    }
}
