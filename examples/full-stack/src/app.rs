use crate::layout::MainLayout;
use crate::pages::{PageBuiltinClubs, PageBuiltinPeople, PageCustomClubs, PageCustomPeople};
use leptos::prelude::*;
use leptos_meta::{Title, provide_meta_context};
use leptos_router::components::{ParentRoute, Redirect, Route, Router, Routes};
use leptos_router::path;

/// The HTML document rendered by the server. The client hydrates its `<body>`.
#[cfg(feature = "ssr")]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    use leptos_meta::MetaTags;

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <link rel="stylesheet" id="leptos" href="/pkg/full-stack.css"/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="CrudKit full-stack example"/>

        <Router>
                <Routes fallback=|| view! { <p class="not-found">"Page not found."</p> }>
                    <ParentRoute path=path!("") view=MainLayout>
                        <Route path=path!("") view=|| view! { <Redirect path="/builtin/clubs"/> }/>
                        <Route path=path!("builtin/clubs") view=PageBuiltinClubs/>
                        <Route path=path!("builtin/people") view=PageBuiltinPeople/>
                        <Route path=path!("custom/clubs") view=PageCustomClubs/>
                        <Route path=path!("custom/people") view=PageCustomPeople/>
                    </ParentRoute>
                </Routes>
        </Router>
    }
}
