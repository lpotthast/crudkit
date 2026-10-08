use crate::ui;
use crudkit_leptos::prelude::*;
use leptos::prelude::*;
use leptos_router::components::Outlet;

/// The application frame around the routed page.
///
/// All pages render below one `CrudInstanceMgr`, which registers mounted instances (required for parent/child
/// instances) and owns the navigation scope used for dirty-form guards.
#[component]
pub fn MainLayout() -> impl IntoView {
    provide_crud_texts(CrudUiTexts::english());

    view! {
        <ui::Shell>
            <CrudInstanceMgr>
                <Outlet />
            </CrudInstanceMgr>
        </ui::Shell>
    }
}
