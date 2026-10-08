//! The example's pages: clubs (with their people nested) and people.

use crate::resources::clubs::{CLUBS_INSTANCE, club_config};
use crate::resources::people::person_config;
use crate::ui;
use leptos::prelude::*;

#[component]
pub fn PageClubs() -> impl IntoView {
    view! {
        <ui::PageHeader
            title="Clubs"
            description="Teams of the league and the people who play for them."
        />
        <ui::Instance name=CLUBS_INSTANCE singular="club" plural="clubs" config=club_config() />
    }
}

#[component]
pub fn PagePeople() -> impl IntoView {
    view! {
        <ui::PageHeader
            title="People"
            description="Everyone registered with a league team, across all teams."
        />
        <ui::Instance name="people" singular="person" plural="people" config=person_config() />
    }
}
