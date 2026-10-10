//! Entity validators of this example.

use crate::server::resources::PersonResource;
use crudkit_rs::crudkit_validation::violation::{Violation, Violations};
use crudkit_rs::prelude::{CrudResource, EntityValidator, ValidationTrigger};
use std::borrow::Cow;

/// Reports a major violation for every person whose gender is not one of the known values.
///
/// Major violations do not block saving. They are persisted and surface as `has_validation_errors` in the read
/// view.
pub struct PersonGenderValidator;

impl PersonGenderValidator {
    fn validate_gender(gender: &str) -> Violations {
        let mut violations = Violations::empty();
        match gender {
            "male" | "female" | "diverse" => {}
            other => violations.push(Violation::major(format!(
                "Gender '{other}' is unknown. Use one of: 'male', 'female', 'diverse'."
            ))),
        }
        violations
    }
}

impl EntityValidator<PersonResource> for PersonGenderValidator {
    fn name(&self) -> Cow<'static, str> {
        "gender".into()
    }

    fn version(&self) -> u32 {
        1
    }

    fn validate_create(
        &self,
        create_model: &<PersonResource as CrudResource>::CreateModel,
        _trigger: ValidationTrigger,
    ) -> Violations {
        Self::validate_gender(&create_model.gender)
    }

    fn validate_model(
        &self,
        model: &<PersonResource as CrudResource>::Model,
        _trigger: ValidationTrigger,
    ) -> Violations {
        Self::validate_gender(&model.gender)
    }

    fn validate_updated(
        &self,
        _old: &<PersonResource as CrudResource>::Model,
        update: &<PersonResource as CrudResource>::UpdateModel,
        _trigger: ValidationTrigger,
    ) -> Violations {
        Self::validate_gender(&update.gender)
    }
}
