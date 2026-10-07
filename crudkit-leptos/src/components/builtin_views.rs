use crate::components::form_views::{CrudCreateView, CrudEditView, CrudReadView};
use crate::components::table_view::CrudTableView;
use crate::config::{CrudViewRegistry, render_error};
use crate::instance::CrudNavigation;
use crudkit_web::view::{CREATE_VIEW, CrudView, EDIT_VIEW, READ_VIEW, TABLE_VIEW};
use leptos::prelude::*;

/// Registers CrudKit's table, create, read, and edit renderers.
pub(crate) fn register_builtin_views(registry: &mut CrudViewRegistry) {
    registry.insert_default(TABLE_VIEW, render_table);
    registry.insert_default(CREATE_VIEW, render_create);
    registry.insert_default(READ_VIEW, render_read);
    registry.insert_default(EDIT_VIEW, render_edit);
}

fn validate(view: &CrudView, subject_required: bool) -> Result<(), String> {
    if !view.payload.is_null() {
        return Err(format!(
            "Built-in view `{}` does not accept a payload",
            view.name
        ));
    }
    match (subject_required, view.subject.is_some()) {
        (true, false) => {
            return Err(format!(
                "Built-in view `{}` requires an entity subject",
                view.name
            ));
        }
        (false, true) => {
            return Err(format!(
                "Built-in view `{}` does not accept an entity subject",
                view.name
            ));
        }
        _ => {}
    }
    Ok(())
}

// View renderers receive the view by value, see `CrudViewRegistry`.
#[allow(clippy::needless_pass_by_value)]
fn render_table(view: CrudView, navigation: CrudNavigation) -> AnyView {
    if let Err(message) = validate(&view, false) {
        return render_error(&view.name, message);
    }
    view! { <CrudTableView navigation /> }.into_any()
}

// View renderers receive the view by value, see `CrudViewRegistry`.
#[allow(clippy::needless_pass_by_value)]
fn render_create(view: CrudView, navigation: CrudNavigation) -> AnyView {
    if let Err(message) = validate(&view, false) {
        return render_error(&view.name, message);
    }
    view! { <CrudCreateView navigation /> }.into_any()
}

fn render_read(view: CrudView, navigation: CrudNavigation) -> AnyView {
    if let Err(message) = validate(&view, true) {
        return render_error(&view.name, message);
    }
    let id = view.subject.expect("built-in read view was validated");
    view! { <CrudReadView id navigation /> }.into_any()
}

fn render_edit(view: CrudView, navigation: CrudNavigation) -> AnyView {
    if let Err(message) = validate(&view, true) {
        return render_error(&view.name, message);
    }
    let id = view.subject.expect("built-in edit view was validated");
    view! { <CrudEditView id navigation /> }.into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;

    #[test]
    fn builtin_subject_requirements_and_payloads_are_validated() {
        assert_that!(validate(&CrudView::table(), false).is_ok()).is_true();
        assert_that!(
            validate(
                &CrudView::edit(crudkit_core::id::SerializableId(Vec::new())),
                true
            )
            .is_ok()
        )
        .is_true();
        assert_that!(validate(&CrudView::new(EDIT_VIEW), true).is_err()).is_true();
        assert_that!(
            validate(
                &CrudView::create().with_payload(serde_json::json!({})),
                false
            )
            .is_err()
        )
        .is_true();
    }
}
