//! The application frame: fonts, sidebar, page header, and notifications.

use crate::ui::icon;
use crudkit_leptos::prelude::*;
use leptonic::atoms::prelude::{
    Toast, ToastCloseButton, ToastContent, ToastDescription, ToastRegion, ToastTitle,
};
use leptonic::hooks::ToastQueue;
use leptos::prelude::*;
use leptos_meta::Link;
use leptos_router::components::A;

const FONTS_URL: &str = "https://fonts.googleapis.com/css2?family=Cinzel:wght@500;600;700\
    &family=EB+Garamond:ital,wght@0,400;0,500;0,600;1,400&display=swap";

/// Wraps every page. CrudKit notifications emitted below are shown as the app's toasts.
#[component]
pub fn Shell(children: Children) -> impl IntoView {
    let toasts = ToastQueue::<CrudNotification>::new(Some(4));
    // Routes CrudKit's notifications into the app's toasts. Each names the instance that sent it, which a page
    // could use to group or filter them.
    provide_crud_notifier(CrudNotifier::toasts(toasts));
    // Every instance below renders with the app's views.
    provide_crud_view_registry(crate::ui::view_registry());

    view! {
        // Display and body typefaces. The stylesheet falls back to local serif fonts while they load.
        <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
        <Link rel="stylesheet" href=FONTS_URL />
        <div class="ui">
            <aside class="ui-sidebar">
                <div class="ui-brand">
                    <span class="ui-brand-mark">{icon(icondata::LuBookMarked)}</span>
                    <span class="ui-brand-text">
                        <strong>"Keeper"</strong>
                        <span>"Quidditch League Registry"</span>
                    </span>
                </div>
                <nav class="ui-nav" aria-label="Keeper">
                    <span class="ui-nav-heading">"The Registry"</span>
                    <A href="/clubs">{icon(icondata::LuShield)}"Clubs"</A>
                    <A href="/people">{icon(icondata::LuUsers)}"People"</A>
                </nav>
                <div class="ui-account">
                    <span class="ui-avatar" aria-hidden="true">
                        "LB"
                    </span>
                    <span class="ui-account-text">
                        <strong>"Ludo Bagman"</strong>
                        <span>"Magical Games & Sports"</span>
                    </span>
                </div>
            </aside>
            <div class="ui-main">{children()}</div>
            <ToastRegion queue=toasts classes="ui-toasts" let:toast>
                {
                    let (tone, tone_icon) = match toast.content.kind {
                        CrudNotificationKind::Success => ("good", icondata::LuSparkles),
                        CrudNotificationKind::Info => ("neutral", icondata::LuFeather),
                        CrudNotificationKind::Warning | CrudNotificationKind::Error => {
                            ("bad", icondata::LuTriangleAlert)
                        }
                    };
                    view! {
                        <Toast toast=toast.clone() classes="ui-toast" attr:data-tone=tone>
                            <span class="ui-toast-icon">{icon(tone_icon)}</span>
                            <ToastContent classes="ui-toast-text">
                                <ToastTitle classes="ui-toast-title">
                                    {toast.content.title.clone()}
                                </ToastTitle>
                                <ToastDescription>{toast.content.message.clone()}</ToastDescription>
                            </ToastContent>
                            <ToastCloseButton classes="ui-toast-close">
                                {icon(icondata::LuX)}
                            </ToastCloseButton>
                        </Toast>
                    }
                }
            </ToastRegion>
        </div>
    }
}

/// Title block at the top of a page.
#[component]
pub fn PageHeader(title: &'static str, description: &'static str) -> impl IntoView {
    view! {
        <header class="ui-page-header">
            <span class="ui-eyebrow">"Ministry of Magic"</span>
            <h1>{title}</h1>
            <p>{description}</p>
        </header>
    }
}
