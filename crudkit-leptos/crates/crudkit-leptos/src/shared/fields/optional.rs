use leptonic::components::prelude::*;
use leptonic::prelude::*;
use leptos::prelude::*;

pub type IsDisabledOrNull = bool;

/// Helper component aiding in implementation of optional fields.
///
/// Shows a small toggle indicating whether the value "is wanted" (should not be `None`) or not
/// wanted (should be `None`).
///
/// The user must enable the toggle to enable the underlying input.
///
/// Whenever the toggle is enabled, `Some(default_provider.run(()))` is chosen as the new value
/// for the field, giving the underlying field is starting value to work with.
///
/// Whenever the toggle is deactivated, `None` is chosen as the new value for the field, erasing
/// any previously stored value.
#[component]
pub fn OptionalInput<T: Clone + Send + Sync + 'static>(
    #[prop(into)] get: Signal<Option<T>>,
    #[prop(into)] set: Out<Option<T>>,
    #[prop(into)] disabled: Signal<bool>,
    #[prop(into)] default_provider: Callback<(), T>,
    #[prop(into)] input_renderer: ViewCallback<Signal<IsDisabledOrNull>>,
) -> impl IntoView {
    // The toggle controls whether the user wants to set a value.
    let is_not_null = Signal::derive(move || get.get().is_some());

    let disabled_or_null = Signal::derive(move || disabled.get() || !is_not_null.get());

    view! {
        <Toggle
            state=is_not_null
            set_state=move |new_state| {
                match new_state {
                    true => {
                        // User wants to set a value. Reset to default.
                        set.set(Some(default_provider.run(())));
                    },
                    false => {
                        // User no longer wants to set a value. Reset to null/None.
                        set.set(None);
                    },
                }
            }
            disabled
            attr:style="font-size: 0.5em; margin-left: 2em;"
        />
        { input_renderer.render(disabled_or_null) }
    }
}
