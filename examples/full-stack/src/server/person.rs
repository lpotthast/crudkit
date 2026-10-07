//! SeaORM entity and CrudKit models of the `people` resource.

use crudkit_rs::prelude::*;
use crudkit_sea_orm::{CkField, CkId, CkSeaOrmBridge, CkSeaOrmCreateModel, CkSeaOrmUpdateModel};
use sea_orm::DerivePrimaryKey;
use sea_orm::EntityTrait;
use sea_orm::EnumIter;
use sea_orm::PrimaryKeyTrait;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    sea_orm::DeriveEntityModel,
    CkId,
    CkField,
    CkSeaOrmBridge,
    CkSeaOrmCreateModel,
    CkSeaOrmUpdateModel,
    crudkit_sea_orm::ReadView,
    ToSchema,
    Serialize,
    Deserialize,
)]
#[sea_orm(table_name = "Person")]
#[read_view(table_name = "PersonReadView")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    #[ck_create_model(exclude)]
    #[ck_update_model(exclude)]
    pub id: i64,

    /// First name of this person.
    pub first_name: String,

    /// Last name of this person.
    pub last_name: String,

    /// Gender of this person. Validated by `PersonGenderValidator`.
    pub gender: String,

    /// Year in which the person was born.
    pub year_of_birth: i32,

    /// The club this person is in.
    pub club_id: i64,

    /// Date and time at which this entity was created.
    pub created_at: time::PrimitiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, sea_orm::DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::club::Entity",
        from = "Column::ClubId",
        to = "super::club::Column::Id"
    )]
    Club,
}

impl sea_orm::ActiveModelBehavior for ActiveModel {}
