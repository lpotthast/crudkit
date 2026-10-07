use crudkit_leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Update model of the `clubs` resource.
#[derive(Clone, PartialEq, Eq, Debug, CkId, CkField, CkResource, Serialize, Deserialize)]
#[ck_resource(resource_name = "clubs")]
#[ck_field(model = Update)]
pub struct Club {
    pub id: i64,

    pub name: String,

    pub created_at: time::PrimitiveDateTime,

    /// Frontend-only field. Its renderer shows the nested people instance.
    #[serde(skip_deserializing)]
    pub people: (),
}

/// Create model of the `clubs` resource.
#[derive(Clone, PartialEq, Eq, Debug, CkField, Serialize, Deserialize)]
#[ck_field(model = Create)]
pub struct CreateClub {
    pub name: String,

    pub created_at: time::PrimitiveDateTime,
}

impl Default for CreateClub {
    fn default() -> Self {
        Self {
            name: String::new(),
            created_at: super::now_primitive(),
        }
    }
}

impl ErasedIdentifiable for CreateClub {
    /// Create models have no identity. CrudKit never asks a create model for its id.
    fn id(&self) -> SerializableId {
        panic!("Create models are not identifiable!")
    }
}

/// Read model of the `clubs` resource, backed by the `ClubReadView` SQL view.
#[derive(Clone, PartialEq, Eq, Debug, CkId, CkField, Serialize, Deserialize)]
#[ck_field(model = Read)]
pub struct ReadClub {
    pub id: i64,

    pub name: String,

    pub created_at: time::PrimitiveDateTime,

    pub has_validation_errors: bool,
}

impl From<ReadClub> for Club {
    fn from(read: ReadClub) -> Self {
        Self {
            id: read.id,
            name: read.name,
            created_at: read.created_at,
            people: (),
        }
    }
}
