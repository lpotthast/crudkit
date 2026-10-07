use crudkit_leptos::prelude::{
    CrudInstanceMgr, CrudNotificationQueue, CrudNotificationRegion, provide_crud_notifier,
};
use leptos::prelude::*;
use leptos_router::components::{A, Outlet};

/// Navigation bar plus the routed page.
///
/// All pages render below one `CrudInstanceMgr`, which registers mounted instances (required for parent/child
/// instances) and owns the navigation scope used for dirty-form guards.
///
/// CrudKit's notifications of every page go to one queue, rendered once for the whole page. Marauder pages route
/// theirs into their own toasts instead.
#[component]
pub fn MainLayout() -> impl IntoView {
    let notifications = CrudNotificationQueue::default();
    provide_crud_notifier(notifications.notifier());

    view! {
        <header class="app-header">
            <span class="app-title">"CrudKit example"</span>
            <nav class="app-nav">
                <span class="app-nav-group">"Built-in UI"</span>
                <A href="/builtin/clubs">"Clubs"</A>
                <A href="/builtin/people">"People"</A>
                <span class="app-nav-group">"Custom UI"</span>
                <A href="/custom/clubs">"Clubs"</A>
                <A href="/custom/people">"People"</A>
            </nav>
        </header>
        <main class="app-content">
            <CrudInstanceMgr>
                <Outlet/>
            </CrudInstanceMgr>
        </main>
        <CrudNotificationRegion queue=notifications />
    }
}
