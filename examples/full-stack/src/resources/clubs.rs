//! The `clubs` resource.

use super::{Ui, disabled_field_options, field_options, header, id_header};
use crate::api::{api_base_url, executor};
use crate::models::club::{Club, ClubField, CreateClub, CrudClubResource, ReadClub};
use crate::models::person::Person;
use crate::resources::people::person_config;
use crudkit_leptos::prelude::*;
use indexmap::indexmap;
use leptos::prelude::*;

/// Instance name of the clubs instance. Referenced by the nested people instance.
pub const CLUBS_INSTANCE: &str = "clubs";

pub fn club_config(ui: Ui) -> CrudInstanceConfig {
    CrudInstanceConfig {
        list_columns: vec![
            id_header(ReadClub::Id),
            header(ReadClub::Name, "Name"),
            header(ReadClub::CreatedAt, "Created at"),
            header(ReadClub::HasValidationErrors, "Validation"),
        ],
        create_elements: CreateElements::Custom(vec![Elem::Enclosing(Enclosing::Card(Group {
            layout: Layout::default(),
            children: vec![Elem::create_field(CreateClub::Name, field_options("Name"))],
        }))]),
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
        read_field_renderer: FieldRendererRegistry::builder()
            .register(ReadClub::HasValidationErrors, ui.validation_status_renderer())
            .build(),
        update_field_renderer: FieldRendererRegistry::builder()
            .register(Club::People, nested_people_renderer(ui))
            .build(),
        ..CrudInstanceConfig::new::<CrudClubResource>(api_base_url(), executor())
    }
}

/// Renders the people of the club being edited as a nested instance in the same UI.
///
/// `CrudParentConfig` scopes the child instance to the parent's id and prefills `club_id` when
/// creating people.
fn nested_people_renderer(ui: Ui) -> FieldRenderer<DynUpdateField> {
    FieldRenderer::new(move |state: CrudFieldState<DynUpdateField>| {
        let parent = Some(CrudParentConfig {
            name: CLUBS_INSTANCE,
            referenced_field: ClubField::Id.name(),
            referencing_field: Person::ClubId.name(),
        });
        let title = state.options.label.clone().map(|label| label.name);
        match ui {
            Ui::Builtin => view! {
                {title.map(|title| view! { <h2 class="nested-title">{title}</h2> })}
                <CrudInstance name="club_people" parent config=person_config(ui) />
            }
            .into_any(),
            Ui::Custom => view! {
                {title.map(|title| view! { <h3 class="mrd-subheading">{title}</h3> })}
                <crate::marauder::MarauderInstance
                    name="club_people"
                    singular="player"
                    plural="players"
                    parent
                    config=person_config(ui)
                />
            }
            .into_any(),
        }
    })
}

/// A resource action reporting its outcome through a CrudKit notification.
fn reload_action() -> CrudAction {
    CrudAction {
        id: "reload",
        name: "Reload".to_owned(),
        icon: None,
        intent: CrudActionIntent::Secondary,
        action: Callback::new(|input: ResourceActionInput| {
            input.and_then.run(Ok(CrudActionAftermath {
                notification: Some(CrudNotification::info("Clubs", "The club list was reloaded.")),
                reload_data: true,
            }));
        }),
        view: None,
    }
}
