//! The app's field inputs: CrudKit's field atoms in the app's markup.

use crudkit_leptos::prelude::*;
use leptonic::atoms::prelude::{Input, TextArea};
use leptos::prelude::*;

/// The input of the surrounding field: its configured renderer, else the app's markup for its kind.
#[component]
pub fn FieldControl() -> impl IntoView {
    view! {
        <CrudFieldControl
            text=|| {
                let multiline = matches!(
                    CrudInputKind::of(use_crud_field_binding().value_kind),
                    CrudInputKind::Text { multiline: true, .. }
                );
                view! {
                    <CrudTextField classes="ui-field">
                        <CrudFieldLabel classes="ui-label" />
                        {if multiline {
                            view! { <TextArea classes="ui-input" /> }.into_any()
                        } else {
                            view! { <Input classes="ui-input" /> }.into_any()
                        }}
                        <CrudFieldError classes="ui-field-error" />
                    </CrudTextField>
                }
            }
            number=|| {
                view! {
                    <CrudNumberField classes="ui-field">
                        <CrudFieldLabel classes="ui-label" />
                        <Input classes="ui-input" />
                        <CrudFieldError classes="ui-field-error" />
                    </CrudNumberField>
                }
            }
            toggle=|| {
                let label = use_crud_field_binding().label();
                view! {
                    <CrudSwitch classes="ui-switch">
                        <span class="ui-switch-track" aria-hidden="true"></span>
                        {label}
                    </CrudSwitch>
                }
            }
        />
    }
}
