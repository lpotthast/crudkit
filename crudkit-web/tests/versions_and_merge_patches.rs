//! The versions that entities carry, and the merge patches of what a change changed.

use assertr::prelude::*;
use crudkit_core::id::{IdValue, SerializableId, SerializableIdEntry};
use crudkit_core_macros::CkId;
use crudkit_web::data_source::{CrudDeleteRequest, CrudUpdateRequest, CrudVersion};
use crudkit_web::model::{DynReadModel, DynReadOrUpdateModel, DynUpdateModel, Model};
use crudkit_web_macros::CkField;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, CkId, CkField, Serialize, Deserialize)]
#[ck_field(model = Update)]
pub struct Wizard {
    pub id: i64,
    pub name: String,
    pub nickname: Option<String>,
    #[ck_field(version)]
    pub version: CrudVersion,
}

#[derive(Clone, PartialEq, Eq, Debug, CkId, CkField, Serialize, Deserialize)]
#[ck_field(model = Read)]
pub struct ReadWizard {
    pub id: i64,
    pub name: String,
    #[ck_field(version)]
    #[serde(default)]
    pub version: Option<CrudVersion>,
}

#[derive(Clone, PartialEq, Eq, Debug, CkId, CkField, Serialize, Deserialize)]
#[ck_field(model = Read)]
pub struct Owl {
    pub id: i64,
}

fn harry(version: &str) -> Wizard {
    Wizard {
        id: 1,
        name: "Harry".to_owned(),
        nickname: Some("The Boy Who Lived".to_owned()),
        version: CrudVersion::new(version),
    }
}

#[test]
fn models_carry_the_version_of_the_field_marked_as_it() {
    let wizard = DynUpdateModel::from(harry("7"));
    let unknown = DynReadModel::from(ReadWizard {
        id: 1,
        name: "Harry".to_owned(),
        version: None,
    });
    let owl = DynReadModel::from(Owl { id: 2 });

    assert_that!(wizard.version()).is_equal_to(Some(&CrudVersion::new("7")));
    assert_that!(unknown.version()).is_none();
    assert_that!(owl.version()).is_none();
}

#[test]
fn versions_are_no_fields_that_forms_show() {
    let names = Wizard::all_fields()
        .iter()
        .map(|field| crudkit_core::Named::name(field).into_owned())
        .collect::<Vec<_>>();

    assert_that!(names).is_equal_to(vec![
        "id".to_owned(),
        "name".to_owned(),
        "nickname".to_owned(),
    ]);
    assert_that!(Wizard::field("version")).is_none();
    // The version is serialized with the model, as an opaque string.
    assert_that!(serde_json::to_value(harry("7")).expect("a wizard")["version"].clone())
        .is_equal_to(serde_json::json!("7"));
}

#[test]
fn updates_are_based_on_the_version_of_their_baseline_and_patch_what_changed() {
    let baseline = harry("7");
    let changed = Wizard {
        name: "Harry Potter".to_owned(),
        nickname: None,
        ..baseline.clone()
    };

    let request = CrudUpdateRequest::new(
        SerializableId(vec![SerializableIdEntry {
            field_name: "id".to_owned(),
            value: IdValue::I64(1),
        }]),
        DynUpdateModel::from(changed),
        DynUpdateModel::from(baseline),
    );

    assert_that!(request.version()).is_equal_to(Some(&CrudVersion::new("7")));
    // A cleared optional field is removed with `null`, and unchanged fields are left out.
    assert_that!(request.merge_patch()).is_equal_to(Ok(serde_json::json!({
        "name": "Harry Potter",
        "nickname": null,
    })));
}

#[test]
fn deletions_are_based_on_the_version_of_the_entity_as_the_user_saw_it() {
    let listed =
        CrudDeleteRequest::new(DynReadOrUpdateModel::from(DynReadModel::from(ReadWizard {
            id: 1,
            name: "Harry".to_owned(),
            version: Some(CrudVersion::new("3")),
        })));
    let edited =
        CrudDeleteRequest::new(DynReadOrUpdateModel::from(DynUpdateModel::from(harry("4"))));

    assert_that!(listed.version()).is_equal_to(Some(&CrudVersion::new("3")));
    assert_that!(edited.version()).is_equal_to(Some(&CrudVersion::new("4")));
}
