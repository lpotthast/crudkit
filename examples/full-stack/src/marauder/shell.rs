//! Application frame for Marauder pages: fonts, sidebar, page header, and Marauder's own notifications.

use crate::marauder::icon;
use crudkit_leptos::prelude::*;
use leptos::prelude::*;
use leptos_meta::Link;
use leptos_router::components::A;
use std::time::Duration;

const FONTS_URL: &str = "https://fonts.googleapis.com/css2?family=Cinzel:wght@500;600;700\
    &family=EB+Garamond:ital,wght@0,400;0,500;0,600;1,400&display=swap";

/// Wraps Marauder pages. CrudKit notifications emitted below are shown as Marauder toasts.
#[component]
pub fn MarauderShell(children: Children) -> impl IntoView {
    let toasts = CrudNotificationQueue::new(Duration::from_secs(5));
    // Routes CrudKit's notifications into the application's own notification UI.
    provide_crud_notifier(toasts.notifier());
    // Every instance below renders with Marauder's views.
    provide_crud_view_registry(crate::marauder::view_registry());

    view! {
        // Display and body typefaces. The stylesheet falls back to local serif fonts while they load.
        <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
        <Link rel="stylesheet" href=FONTS_URL />
        <div class="mrd">
            <aside class="mrd-sidebar">
                <div class="mrd-brand">
                    <span class="mrd-brand-mark">{icon(icondata::LuMap)}</span>
                    <span class="mrd-brand-text">
                        <strong>"Marauder"</strong>
                        <span>"Quidditch League Registry"</span>
                    </span>
                </div>
                <nav class="mrd-nav" aria-label="Marauder">
                    <span class="mrd-nav-heading">"The Registry"</span>
                    <A href="/custom/clubs">{icon(icondata::LuShield)}"Clubs"</A>
                    <A href="/custom/people">{icon(icondata::LuUsers)}"People"</A>
                </nav>
                <div class="mrd-account">
                    <span class="mrd-avatar" aria-hidden="true">
                        "LB"
                    </span>
                    <span class="mrd-account-text">
                        <strong>"Ludo Bagman"</strong>
                        <span>"Magical Games & Sports"</span>
                    </span>
                </div>
            </aside>
            <div class="mrd-main">{children()}</div>
            <ul class="mrd-toasts" role="status" aria-live="polite">
                <For
                    each=move || toasts.entries.get()
                    key=|(id, _)| *id
                    children=move |(id, notification)| {
                        let (tone, tone_icon) = match notification.kind {
                            CrudNotificationKind::Success => ("good", icondata::LuSparkles),
                            CrudNotificationKind::Info => ("neutral", icondata::LuFeather),
                            CrudNotificationKind::Warning | CrudNotificationKind::Error => {
                                ("bad", icondata::LuTriangleAlert)
                            }
                        };
                        view! {
                            <li class="mrd-toast" data-tone=tone>
                                <span class="mrd-toast-icon">{icon(tone_icon)}</span>
                                <span class="mrd-toast-text">
                                    <strong>{notification.title}</strong>
                                    <span>{notification.message}</span>
                                </span>
                                <button
                                    type="button"
                                    class="mrd-toast-close"
                                    on:click=move |_| toasts.dismiss(id)
                                >
                                    {icon(icondata::LuX)}
                                    <span class="mrd-sr-only">"Dismiss"</span>
                                </button>
                            </li>
                        }
                    }
                />
            </ul>
        </div>
    }
}

/// Title block at the top of a Marauder page.
#[component]
pub fn MarauderPageHeader(title: &'static str, description: &'static str) -> impl IntoView {
    view! {
        <header class="mrd-page-header">
            <span class="mrd-eyebrow">"Ministry of Magic"</span>
            <h1>{title}</h1>
            <p>{description}</p>
        </header>
    }
}
