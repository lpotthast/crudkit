use crate::crud_create_view::CrudCreateView;
use crate::crud_edit_view::CrudEditView;
use crate::crud_instance::CrudInstanceContext;
use crate::crud_list_view::CrudListView;
use crate::crud_navigation::CrudNavigation;
use crate::crud_read_view::CrudReadView;
use crate::crud_view_registry::{CrudViewRegistry, render_error};
use crudkit_web::request_error::CrudOperationError;
use crudkit_web::view::{CREATE_VIEW, CrudView, EDIT_VIEW, READ_VIEW, TABLE_VIEW};
use leptonic::components::prelude::*;
use leptos::prelude::*;
use time::OffsetDateTime;
use uuid::Uuid;

impl Default for CrudViewRegistry {
    fn default() -> Self {
        let mut registry = Self::empty();
        registry.insert_default(TABLE_VIEW, render_table);
        registry.insert_default(CREATE_VIEW, render_create);
        registry.insert_default(READ_VIEW, render_read);
        registry.insert_default(EDIT_VIEW, render_edit);
        registry
    }
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

fn render_table(view: CrudView, navigation: CrudNavigation) -> AnyView {
    if let Err(message) = validate(&view, false) {
        return render_error(&view.name, message);
    }
    let ctx = expect_context::<CrudInstanceContext>();
    let static_config = ctx.static_config;
    let field_renderer_registry =
        Signal::derive(move || static_config.read_value().read_field_renderer.clone());
    let actions = Signal::derive(move || static_config.read_value().actions.clone());

    view! {
        <CrudListView
            data_provider=ctx.data_provider
            headers=ctx.headers
            order_by=ctx.order_by
            field_renderer_registry=field_renderer_registry
            actions=actions
            navigation=navigation
        />
    }
    .into_any()
}

fn render_create(view: CrudView, navigation: CrudNavigation) -> AnyView {
    if let Err(message) = validate(&view, false) {
        return render_error(&view.name, message);
    }
    let ctx = expect_context::<CrudInstanceContext>();
    let static_config = ctx.static_config;
    let field_renderer_registry =
        Signal::derive(move || static_config.read_value().create_field_renderer.clone());
    let controls = Signal::derive(move || static_config.read_value().builtin_view_controls);

    view! {
        <CrudCreateView
            data_provider=ctx.data_provider
            create_elements=ctx.create_elements
            field_renderer_registry=field_renderer_registry
            controls=controls
            navigation=navigation
            on_entity_created=move |_saved| {}
            on_entity_creation_failed=move |error: CrudOperationError| {
                expect_context::<Toasts>()
                    .push(Toast {
                        id: Uuid::new_v4(),
                        created_at: OffsetDateTime::now_utc(),
                        variant: ToastVariant::Error,
                        header: ViewFn::from(|| "Fehler"),
                        body: ViewFn::from(move || {
                            format!("Eintrag konnte nicht erstellt werden.\n{error}")
                        }),
                        timeout: ToastTimeout::DefaultDelay,
                    })
            }
            on_tab_selected=move |tab_id| ctx.tab_selected(tab_id)
        />
    }
    .into_any()
}

fn render_read(view: CrudView, navigation: CrudNavigation) -> AnyView {
    if let Err(message) = validate(&view, true) {
        return render_error(&view.name, message);
    }
    let ctx = expect_context::<CrudInstanceContext>();
    let id = view.subject.expect("built-in read view was validated");
    let static_config = ctx.static_config;
    let actions = Signal::derive(move || static_config.read_value().entity_actions.clone());
    let field_renderer_registry =
        Signal::derive(move || static_config.read_value().update_field_renderer.clone());
    let controls = Signal::derive(move || static_config.read_value().builtin_view_controls);

    view! {
        <CrudReadView
            id=id
            data_provider=ctx.data_provider
            actions=actions
            elements=ctx.update_elements
            field_renderer_registry=field_renderer_registry
            controls=controls
            navigation=navigation
            on_tab_selected=move |tab_id| ctx.tab_selected(tab_id)
        />
    }
    .into_any()
}

fn render_edit(view: CrudView, navigation: CrudNavigation) -> AnyView {
    if let Err(message) = validate(&view, true) {
        return render_error(&view.name, message);
    }
    let ctx = expect_context::<CrudInstanceContext>();
    let id = view.subject.expect("built-in edit view was validated");
    let static_config = ctx.static_config;
    let actions = Signal::derive(move || static_config.read_value().entity_actions.clone());
    let field_renderer_registry =
        Signal::derive(move || static_config.read_value().update_field_renderer.clone());
    let controls = Signal::derive(move || static_config.read_value().builtin_view_controls);

    view! {
        <CrudEditView
            id=id
            data_provider=ctx.data_provider
            actions=actions
            elements=ctx.update_elements
            field_renderer_registry=field_renderer_registry
            controls=controls
            navigation=navigation
            on_entity_updated=move |_saved| {}
            on_entity_update_failed=move |_error: CrudOperationError| {}
            on_tab_selected=move |tab_id| ctx.tab_selected(tab_id)
        />
    }
    .into_any()
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
