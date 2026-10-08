use crate::layout::MainLayout;
use crate::pages::{PageClubs, PagePeople};
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
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <link rel="stylesheet" id="leptos" href="/pkg/full-stack.css" />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="CrudKit full-stack example" />

        <Router>
            <Routes fallback=|| view! { <p class="not-found">"Page not found."</p> }>
                <ParentRoute path=path!("") view=MainLayout>
                    <Route path=path!("") view=|| view! { <Redirect path="/clubs" /> } />
                    <Route path=path!("clubs") view=PageClubs />
                    <Route path=path!("people") view=PagePeople />
                </ParentRoute>
            </Routes>
        </Router>
    }
}
