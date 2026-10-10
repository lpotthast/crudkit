//! The `clubs` resource.

use super::{disabled_field_options, field_options, header, id_header};
use crate::api::{api_base_url, executor};
use crate::models::club::{Club, ClubField, CreateClub, CrudClubResource, ReadClub};
use crate::models::person::Person;
use crate::resources::people::person_config;
use crate::ui;
use crudkit_leptos::prelude::*;
use indexmap::indexmap;
use leptos::prelude::*;

/// Instance name of the clubs instance. Referenced by the nested people instance.
pub const CLUBS_INSTANCE: &str = "clubs";

pub fn club_config() -> CrudInstanceConfig {
    CrudInstanceConfig {
        list_columns: vec![
            id_header(ReadClub::Id),
            header(ReadClub::Name, "Name"),
            header(ReadClub::CreatedAt, "Created at"),
            header(ReadClub::HasValidationErrors, "Validation"),
        ],
        create_elements: vec![Elem::Enclosing(Enclosing::Card(Group {
            layout: Layout::default(),
            children: vec![Elem::create_field(CreateClub::Name, field_options("Name"))],
        }))],
        elements: vec![Elem::Enclosing(Enclosing::Card(Group {
            layout: Layout::default(),
            children: vec![
                Elem::field(Club::Id, disabled_field_options("ID")),
                Elem::field(Club::Name, field_options("Name")),
                Elem::field(Club::CreatedAt, disabled_field_options("Created at")),
                Elem::field(Club::People, field_options("People")),
            ],
        }))],
        order_by: indexmap! {
            ReadClub::Id.into() => Order::Asc,
        },
        actions: vec![reload_action()],
        read_field_renderer: FieldRendererRegistry::default().register(
            ReadClub::HasValidationErrors,
            ui::validation_status_renderer(),
        ),
        update_field_renderer: FieldRendererRegistry::default()
            .register(Club::People, nested_people_renderer()),
        ..CrudInstanceConfig::new::<CrudClubResource>(api_base_url(), executor())
    }
}

/// Renders the people of the club being edited as a nested instance.
///
/// `CrudParentConfig` scopes the child instance to the parent's id and prefills `club_id` when
/// creating people.
fn nested_people_renderer() -> FieldRenderer<DynUpdateField> {
    FieldRenderer::new(|state: CrudFieldState<DynUpdateField>| {
        let parent = Some(CrudParentConfig {
            name: CLUBS_INSTANCE,
            referenced_field: ClubField::Id.name(),
            referencing_field: Person::ClubId.name(),
        });
        let title = state.options.label.clone().map(|label| label.name);
        // Outside of editing the club, e.g. in its read view, its players are only shown.
        let read_only = state.mode != FieldMode::Editable;
        view! {
            {title.map(|title| view! { <h3 class="ui-subheading">{title}</h3> })}
            <ui::Instance
                name="club_people"
                singular="player"
                plural="players"
                parent
                read_only
                config=person_config()
            />
        }
    })
}

/// A resource action reporting its outcome through a CrudKit notification.
fn reload_action() -> CrudAction {
    CrudAction {
        id: "reload",
        name: "Reload".to_owned(),
        icon: Some(icondata::LuRefreshCw),
        intent: CrudActionIntent::Secondary,
        action: Callback::new(|input: ResourceActionInput| {
            input.and_then.run(Ok(CrudActionAftermath {
                notification: Some(CrudNotification::info(
                    "Clubs",
                    "The club list was reloaded.",
                )),
                reload_data: true,
            }));
        }),
        view: None,
    }
}
