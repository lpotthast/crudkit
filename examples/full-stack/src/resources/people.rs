//! The `people` resource.

use super::{Ui, disabled_field_options, field_options, header, id_header};
use crate::api::{api_base_url, executor};
use crate::models::person::{CreatePerson, CrudPersonResource, Person, ReadPerson};
use crudkit_leptos::prelude::*;
use indexmap::indexmap;

/// Configuration of a `people` instance. Used standalone and nested below a club.
pub fn person_config(ui: Ui) -> CrudInstanceConfig {
    CrudInstanceConfig {
        list_columns: vec![
            id_header(ReadPerson::Id),
            header(ReadPerson::FirstName, "First name"),
            header(ReadPerson::LastName, "Last name"),
            header(ReadPerson::Gender, "Gender"),
            header(ReadPerson::YearOfBirth, "Year of birth"),
            header(ReadPerson::ClubId, "Club"),
            header(ReadPerson::CreatedAt, "Created at"),
            header(ReadPerson::HasValidationErrors, "Validation"),
        ],
        create_elements: CreateElements::Custom(vec![Elem::Enclosing(Enclosing::Card(Group {
            layout: Layout::default(),
            children: vec![
                Elem::create_field(CreatePerson::FirstName, field_options("First name")),
                Elem::create_field(CreatePerson::LastName, field_options("Last name")),
                Elem::create_field(CreatePerson::Gender, field_options("Gender")),
                Elem::create_field(CreatePerson::YearOfBirth, field_options("Year of birth")),
                Elem::create_field(CreatePerson::ClubId, field_options("Club")),
            ],
        }))]),
        // Tabs keep their selection while switching between reading and editing a person.
        elements: vec![
            Elem::Enclosing(Enclosing::Card(Group {
                layout: Layout::Columns2,
                children: vec![
                    Elem::field(Person::Id, disabled_field_options("ID")),
                    Elem::field(Person::CreatedAt, disabled_field_options("Created at")),
                ],
            })),
            Elem::Enclosing(Enclosing::Tabs(vec![
                Tab {
                    id: "personal".into(),
                    label: Label::new("Personal"),
                    group: Group {
                        layout: Layout::Columns2,
                        children: vec![
                            Elem::field(Person::FirstName, field_options("First name")),
                            Elem::field(Person::LastName, field_options("Last name")),
                            Elem::field(Person::Gender, field_options("Gender")),
                            Elem::field(Person::YearOfBirth, field_options("Year of birth")),
                        ],
                    },
                },
                Tab {
                    id: "membership".into(),
                    label: Label::new("Membership"),
                    group: Group {
                        layout: Layout::default(),
                        children: vec![Elem::field(Person::ClubId, field_options("Club"))],
                    },
                },
            ])),
        ],
        order_by: indexmap! {
            ReadPerson::Id.into() => Order::Asc,
        },
        read_field_renderer: FieldRendererRegistry::builder()
            .register(ReadPerson::HasValidationErrors, ui.validation_status_renderer())
            .build(),
        ..CrudInstanceConfig::new::<CrudPersonResource>(api_base_url(), executor())
    }
}
