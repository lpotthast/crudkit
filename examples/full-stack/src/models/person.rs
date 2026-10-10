use crudkit_leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Update model of the `people` resource.
#[derive(Clone, PartialEq, Eq, Debug, CkId, CkField, CkResource, Serialize, Deserialize)]
#[ck_resource(resource_name = "people")]
#[ck_field(model = Update)]
pub struct Person {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub gender: String,
    pub year_of_birth: i32,
    pub club_id: i64,
    pub created_at: time::PrimitiveDateTime,
}

/// Create model of the `people` resource.
#[derive(Clone, PartialEq, Eq, Debug, CkField, Serialize, Deserialize)]
#[ck_field(model = Create)]
pub struct CreatePerson {
    pub first_name: String,
    pub last_name: String,
    pub gender: String,
    pub year_of_birth: i32,
    pub club_id: i64,
    pub created_at: time::PrimitiveDateTime,
}

impl Default for CreatePerson {
    fn default() -> Self {
        Self {
            first_name: String::new(),
            last_name: String::new(),
            gender: String::new(),
            year_of_birth: 2000,
            club_id: 0,
            created_at: super::now_primitive(),
        }
    }
}

impl ErasedIdentifiable for CreatePerson {
    /// Create models have no identity. CrudKit never asks a create model for its id.
    fn id(&self) -> SerializableId {
        panic!("Create models are not identifiable!")
    }
}

/// Read model of the `people` resource, backed by the `PersonReadView` SQL view.
#[derive(Clone, PartialEq, Eq, Debug, CkId, CkField, Serialize, Deserialize)]
#[ck_field(model = Read)]
pub struct ReadPerson {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub gender: String,
    pub year_of_birth: i32,
    pub club_id: i64,
    pub created_at: time::PrimitiveDateTime,
    pub has_validation_errors: bool,
}

impl From<ReadPerson> for Person {
    fn from(read: ReadPerson) -> Self {
        Self {
            id: read.id,
            first_name: read.first_name,
            last_name: read.last_name,
            gender: read.gender,
            year_of_birth: read.year_of_birth,
            club_id: read.club_id,
            created_at: read.created_at,
        }
    }
}
