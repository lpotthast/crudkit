//! The example's pages: each resource once with CrudKit's built-in UI and once with Marauder's own UI.

use crate::marauder::{MarauderInstance, MarauderPageHeader, MarauderShell};
use crate::resources::Ui;
use crate::resources::clubs::{CLUBS_INSTANCE, club_config};
use crate::resources::people::person_config;
use crudkit_leptos::prelude::*;
use leptos::prelude::*;

#[component]
pub fn PageBuiltinClubs() -> impl IntoView {
    view! {
        <h1>"Clubs"</h1>
        <CrudInstance name=CLUBS_INSTANCE config=club_config(Ui::Builtin) />
    }
}

#[component]
pub fn PageBuiltinPeople() -> impl IntoView {
    view! {
        <h1>"People"</h1>
        <CrudInstance name="people" config=person_config(Ui::Builtin) />
    }
}

#[component]
pub fn PageCustomClubs() -> impl IntoView {
    view! {
        <MarauderShell>
            <MarauderPageHeader
                title="Clubs"
                description="Teams of the league and the people who play for them."
            />
            <MarauderInstance
                name=CLUBS_INSTANCE
                singular="club"
                plural="clubs"
                config=club_config(Ui::Custom)
            />
        </MarauderShell>
    }
}

#[component]
pub fn PageCustomPeople() -> impl IntoView {
    view! {
        <MarauderShell>
            <MarauderPageHeader
                title="People"
                description="Everyone registered with a league team, across all teams."
            />
            <MarauderInstance
                name="people"
                singular="person"
                plural="people"
                config=person_config(Ui::Custom)
            />
        </MarauderShell>
    }
}
