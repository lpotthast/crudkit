//! The frontend `CkResource` and `CkField` derives on a minimal resource.

use assertr::prelude::*;
use crudkit_core::id::{ErasedIdentifiable, SerializableId};
use crudkit_core_macros::CkId;
use crudkit_web::action::ActionPayload;
use crudkit_web::model::Model;
use crudkit_web::resource::Resource;
use crudkit_web_macros::{CkActionPayload, CkField, CkResource};
use serde::{Deserialize, Serialize};

mod payloads {
    use super::{CkActionPayload, Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, CkActionPayload)]
    pub struct Archive {
        pub reason: String,
    }
}

#[derive(Clone, PartialEq, Eq, Debug, CkId, CkField, CkResource, Serialize, Deserialize)]
#[ck_resource(resource_name = "wizards", action_payload = payloads::Archive)]
#[ck_field(model = Update)]
pub struct Wizard {
    pub id: i64,
    pub house: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Default, CkField, Serialize, Deserialize)]
#[ck_field(model = Create)]
pub struct CreateWizard {
    pub house: String,
}

impl ErasedIdentifiable for CreateWizard {
    fn id(&self) -> SerializableId {
        unreachable!("create models are not identifiable")
    }
}

#[derive(Clone, PartialEq, Eq, Debug, CkId, CkField, Serialize, Deserialize)]
#[ck_field(model = Read)]
pub struct ReadWizard {
    pub id: i64,
    pub house: String,
}

impl From<ReadWizard> for Wizard {
    fn from(read: ReadWizard) -> Self {
        Self {
            id: read.id,
            house: read.house,
        }
    }
}

fn action_payload_of<R: Resource>() -> std::any::TypeId
where
    R::ActionPayload: 'static,
{
    std::any::TypeId::of::<R::ActionPayload>()
}

fn assert_action_payload<P: ActionPayload>() {}

#[test]
fn resource_uses_the_action_payload_given_as_a_type_path() {
    assert_action_payload::<payloads::Archive>();
    assert_that!(CrudWizardResource::resource_name()).is_equal_to("wizards");
    assert_that!(action_payload_of::<CrudWizardResource>())
        .is_equal_to(std::any::TypeId::of::<payloads::Archive>());
}

#[test]
fn fields_are_looked_up_by_name_without_panicking() {
    assert_that!(ReadWizard::field("house")).is_equal_to(Some(ReadWizardField::House));
    assert_that!(ReadWizard::field("hat")).is_none();
}

fn models() -> crudkit_web::model_handler::ModelHandler {
    crudkit_web::model_handler::ModelHandler::new::<CreateWizard, ReadWizard, Wizard>()
}

#[test]
fn model_handler_decodes_and_converts_models_of_its_resource() {
    let read = models()
        .deserialize_read_model(serde_json::json!({ "id": 3, "house": "Ravenclaw" }))
        .expect("read model should deserialize");
    assert_that!(read.downcast_ref::<ReadWizard>().house.clone())
        .is_equal_to("Ravenclaw".to_owned());

    let update = models().read_model_to_update_model(read);
    assert_that!(update.downcast_ref::<Wizard>().id).is_equal_to(3);

    // `Value` has no `PartialEq`, so the values are compared by their debug rendering.
    let values = models()
        .update_model_values(&update)
        .into_iter()
        .map(|(field, value)| {
            (
                crudkit_core::Named::name(&field).into_owned(),
                format!("{value:?}"),
            )
        })
        .collect::<Vec<_>>();
    assert_that!(values).is_equal_to(vec![
        ("id".to_owned(), "I64(3)".to_owned()),
        ("house".to_owned(), "String(\"Ravenclaw\")".to_owned()),
    ]);
}

#[test]
fn model_handler_looks_up_create_fields_and_builds_defaults() {
    assert_that!(models().create_model_field("house").is_some()).is_true();
    assert_that!(models().create_model_field("wand").is_none()).is_true();
    let default = models().default_create_model();
    assert_that!(default.downcast_ref::<CreateWizard>().clone())
        .is_equal_to(CreateWizard::default());
}

#[test]
fn model_handler_reports_json_of_another_shape() {
    assert_that!(
        models()
            .deserialize_update_model(serde_json::json!({ "id": "three" }))
            .is_err()
    )
    .is_true();
}
