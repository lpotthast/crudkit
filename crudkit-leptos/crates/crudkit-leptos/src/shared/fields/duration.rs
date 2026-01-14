use crate::shared::fields::optional::OptionalInput;
use crate::shared::fields::render_label;
use crudkit_core::{TimeDuration, Value};
use crudkit_web::{FieldMode, FieldOptions};
use leptonic::Out;
use leptonic::components::input::NumberInput;
use leptos::prelude::*;
use std::sync::Arc;

fn with_hours(duration: time::Duration, hours: i64) -> time::Duration {
    let minutes = duration.whole_minutes() % 60;
    let seconds = duration.whole_seconds() % 60;
    let subsec_milliseconds = duration.subsec_milliseconds() as i64;

    time::Duration::milliseconds(
        hours * 3_600_000 + minutes * 60_000 + seconds * 1_000 + subsec_milliseconds,
    )
}

fn with_minutes(duration: time::Duration, minutes: i64) -> time::Duration {
    let hours = duration.whole_hours();
    let seconds = duration.whole_seconds() % 60;
    let subsec_milliseconds = duration.subsec_milliseconds() as i64;

    time::Duration::milliseconds(
        hours * 3_600_000 + minutes * 60_000 + seconds * 1_000 + subsec_milliseconds,
    )
}

fn with_seconds(duration: time::Duration, seconds: i64) -> time::Duration {
    let hours = duration.whole_hours();
    let minutes = duration.whole_minutes() % 60;
    let subsec_milliseconds = duration.subsec_milliseconds() as i64;

    time::Duration::milliseconds(
        hours * 3_600_000 + minutes * 60_000 + seconds * 1_000 + subsec_milliseconds,
    )
}

fn with_centiseconds(duration: time::Duration, centiseconds: i64) -> time::Duration {
    let hours = duration.whole_hours();
    let minutes = duration.whole_minutes() % 60;
    let seconds = duration.whole_seconds() % 60;

    time::Duration::milliseconds(
        hours * 3_600_000 + minutes * 60_000 + seconds * 1_000 + centiseconds * 10,
    )
}

fn duration_to_string(duration: time::Duration) -> String {
    // Format components manually since time::Duration doesn't have Display
    let hours = duration.whole_hours();
    let minutes = duration.whole_minutes() % 60;
    let seconds = duration.whole_seconds() % 60;
    let centiseconds = (duration.whole_milliseconds() % 1000) / 10;

    format!(
        "{:02}:{:02}:{:02}.{:02}",
        hours, minutes, seconds, centiseconds
    )
}

#[component]
fn DurationInput(
    #[prop(into)] get: Signal<time::Duration>,
    #[prop(into, optional)] set: Option<Out<time::Duration>>,
    #[prop(into, optional)] disabled: Signal<bool>,
) -> impl IntoView {
    let get_hours = Signal::derive(move || get.get().whole_hours() as f64);
    let get_minutes = Signal::derive(move || (get.get().whole_minutes() % 60) as f64);
    let get_seconds = Signal::derive(move || (get.get().whole_seconds() % 60) as f64);
    let get_centiseconds = Signal::derive(move || (get.get().subsec_milliseconds() / 10) as f64);

    let last_error = RwSignal::<Option<String>>::new(None);

    let set_hours = {
        move |hours: i64| {
            if let Some(set) = set {
                let current = get.get();
                let new = with_hours(current, hours);
                set.set(new);
            }
        }
    };
    let set_minutes = {
        move |minutes: i64| {
            if let Some(set) = set {
                let current = get.get();
                let new = with_minutes(current, minutes);
                set.set(new);
            }
        }
    };
    let set_seconds = {
        move |seconds: i64| {
            if let Some(set) = set {
                let current = get.get();
                let new = with_seconds(current, seconds);
                set.set(new);
            }
        }
    };
    let set_centiseconds = {
        move |centiseconds: i64| {
            if let Some(set) = set {
                let current = get.get();
                let new = with_centiseconds(current, centiseconds);
                set.set(new);
            }
        }
    };

    view! {
        <div style="display: flex; flex-direction: row;">
            // Hours
            <NumberInput
                // TODO: This should not be necessary. We can style the leptonic-input directly.
                attr:class="crud-input-field"
                attr:pattern="[0-9]*"
                min=0.0
                max=99.0
                step=1.0
                get=get_hours
                set=move |hours: f64| set_hours(hours as i64)
                disabled=disabled
            />

            <span style="
                font-weight: 900;
                text-align: center;
                line-height: 2em;
                padding: 0.2em;"
            >":"</span>

            // Minutes
            <NumberInput
                // TODO: This should not be necessary. We can style the leptonic-input directly.
                attr:class="crud-input-field"
                attr:pattern="[0-9]*"
                min=0.0
                max=59.0
                step=1.0
                get=get_minutes
                set=move |minutes: f64| set_minutes(minutes as i64)
                disabled=disabled
            />

            <span style="
                font-weight: 900;
                text-align: center;
                line-height: 2em;
                padding: 0.2em;"
            >":"</span>

            // Seconds
            <NumberInput
                // TODO: This should not be necessary. We can style the leptonic-input directly.
                attr:class="crud-input-field"
                attr:pattern="[0-9]*"
                min=0.0
                max=59.0
                step=1.0
                get=get_seconds
                set=move |seconds: f64| set_seconds(seconds as i64)
                disabled=disabled
            />

            <span style="
                font-weight: 900;
                text-align: center;
                line-height: 2em;
                padding: 0.2em;"
            >","</span>

            // Centiseconds
            <NumberInput
                // TODO: This should not be necessary. We can style the leptonic-input directly.
                attr:class="crud-input-field"
                attr:pattern="[0-9]*"
                min=0.0
                max=99.0
                step=1.0
                get=get_centiseconds
                set=move |centiseconds: f64| set_centiseconds(centiseconds as i64)
                disabled=disabled
            />

            <div>
                <Show
                  when=move || { last_error.get().is_some() }
                  fallback=|| view! { }
                >
                    { last_error.get().expect("present") }
                </Show>
            </div>
        </div>
    }
}

#[component]
pub fn OptionalDurationInput(
    #[prop(into)] get: Signal<Option<time::Duration>>,
    #[prop(into)] set: Out<Option<time::Duration>>,
    #[prop(into, optional)] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <OptionalInput
            set=set
            get=get
            disabled
            default_provider=move || time::Duration::ZERO
            input_renderer=move |disabled_or_null| view! {
                <DurationInput
                    disabled=disabled_or_null
                    get=Signal::derive(move || get.get().unwrap_or(time::Duration::ZERO))
                    set=Callback::new(move |new: time::Duration| {
                        set.set(Some(new));
                    })
                />
            }
        />
    }
}

#[component]
pub fn CrudDurationField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<TimeDuration>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => {
            view! { <div>{move || duration_to_string(value.get().0) }</div> }.into_any()
        }
        FieldMode::Readable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <DurationInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=true
                    get=Signal::derive(move || value.get().0)
                />
            </div>
        }
        .into_any(),
        FieldMode::Editable => view! {
            <div class="crud-field">
                {render_label(field_options.label.clone())}
                <DurationInput
                    attr:id=id.clone()
                    // TODO: This should not be necessary. We can style the leptonic-input directly.
                    attr:class="crud-input-field"
                    disabled=false
                    get=Signal::derive(move || value.get().0)
                    set=move |new: time::Duration| value_changed.run(Ok(Value::Duration(TimeDuration(new))))
                />
            </div>
        }
        .into_any(),
    }
}

#[component]
pub fn CrudOptionalDurationField(
    id: String,
    field_options: FieldOptions,
    field_mode: FieldMode,
    #[prop(into)] value: Signal<Option<TimeDuration>>,
    value_changed: Callback<Result<Value, Arc<dyn std::error::Error>>>,
) -> impl IntoView {
    match field_mode {
        FieldMode::Display => {
            move || match value.get() {
                Some(value) => view! { <div>{duration_to_string(value.0)}</div> }.into_any(),
                None => view! { <div>"-"</div> }.into_any(),
            }
        }
        .into_any(),
        FieldMode::Readable => {
            view! {
                <div class="crud-field">
                    {render_label(field_options.label.clone())}
                    <OptionalDurationInput
                        attr:id=id.clone()
                        // TODO: This should not be necessary. We can style the leptonic-input directly.
                        attr:class="crud-input-field"
                        disabled=false
                        get=Signal::derive(move || { value.get().map(|it| it.0) })
                        set={move |_new: Option<time::Duration>| {}}
                    />
                </div>
            }
            .into_any()
        }
        FieldMode::Editable => {
            view! {
                <div class="crud-field">
                    {render_label(field_options.label.clone())}
                    <OptionalDurationInput
                        attr:id=id.clone()
                        // TODO: This should not be necessary. We can style the leptonic-input directly.
                        attr:class="crud-input-field"
                        disabled=field_options.disabled
                        get=Signal::derive(move || { value.get().map(|it| it.0) })
                        set={move |new: Option<time::Duration>| {
                            value_changed.run(Ok(Value::OptionalDuration(new.map(|it| TimeDuration(it)))))
                        }}
                    />
                </div>
            }
            .into_any()
        }
    }
}
