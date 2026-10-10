//! CrudKit resource definitions binding the SeaORM entities to crudkit-rs.

use crate::server::collaboration::NoopCollaborationService;
use crate::server::{club, person};
use crudkit_rs::lifetime::NoopLifetimeHooks;
use crudkit_rs::prelude::*;
use crudkit_sea_orm::repo::SeaOrmRepo;
use crudkit_sea_orm::validation::unified::repository::UnifiedValidationRepository;
use crudkit_sea_orm::{CrudColumns, SeaOrmResource};
use utoipa::ToSchema;

/// All resources served by this example.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resources {
    Club,
    Person,
}

impl ResourceType for Resources {
    /// The URL segment of each resource. Must match the frontend `ck_resource(resource_name)`.
    fn name(&self) -> &'static str {
        match self {
            Resources::Club => "clubs",
            Resources::Person => "people",
        }
    }
}

#[derive(Debug, CkResourceContext)]
pub struct ClubResourceContext;

#[derive(Debug, ToSchema)]
pub struct ClubResource;

impl CrudResource for ClubResource {
    type ReadModel = club::read_view::Model;
    type ReadModelId = club::read_view::ModelId;
    type ReadModelField = club::read_view::ModelField;

    type CreateModel = club::CreateModel;
    type CreateModelField = club::ModelField;

    type UpdateModel = club::UpdateModel;
    type UpdateModelField = club::ModelField;

    type Model = club::Model;
    type Id = club::ModelId;
    type ModelField = club::ModelField;

    type Repository = SeaOrmRepo;
    type ValidationResultRepository = UnifiedValidationRepository;
    type CollaborationService = NoopCollaborationService;
    type Context = ClubResourceContext;
    type HookData = ();
    type Lifetime = NoopLifetimeHooks;
    type Auth = NoAuth;
    type AuthPolicy = OpenAuthPolicy;
    type ResourceType = Resources;
    const TYPE: Resources = Resources::Club;
}

impl SeaOrmResource for ClubResource {
    type Entity = club::Entity;
    type SeaOrmModel = club::Model;
    type ActiveModel = club::ActiveModel;
    type Column = club::Column;
    type PrimaryKey = <club::Entity as sea_orm::EntityTrait>::PrimaryKey;

    type ReadViewEntity = club::read_view::Entity;
    type ReadViewSeaOrmModel = club::read_view::Model;
    type ReadViewActiveModel = club::read_view::ActiveModel;
    type ReadViewColumn = club::read_view::Column;
    type ReadViewPrimaryKey = <club::read_view::Entity as sea_orm::EntityTrait>::PrimaryKey;

    fn model_field_to_column(field: &Self::ModelField) -> Self::Column {
        <club::ModelField as CrudColumns<club::Column>>::to_sea_orm_column(field)
    }

    fn read_model_field_to_column(field: &Self::ReadModelField) -> Self::ReadViewColumn {
        <club::read_view::ModelField as CrudColumns<club::read_view::Column>>::to_sea_orm_column(
            field,
        )
    }
}

#[derive(Debug, CkResourceContext)]
pub struct PersonResourceContext;

#[derive(Debug, ToSchema)]
pub struct PersonResource;

impl CrudResource for PersonResource {
    type ReadModel = person::read_view::Model;
    type ReadModelId = person::read_view::ModelId;
    type ReadModelField = person::read_view::ModelField;

    type CreateModel = person::CreateModel;
    type CreateModelField = person::ModelField;

    type UpdateModel = person::UpdateModel;
    type UpdateModelField = person::ModelField;

    type Model = person::Model;
    type Id = person::ModelId;
    type ModelField = person::ModelField;

    type Repository = SeaOrmRepo;
    type ValidationResultRepository = UnifiedValidationRepository;
    type CollaborationService = NoopCollaborationService;
    type Context = PersonResourceContext;
    type HookData = ();
    type Lifetime = NoopLifetimeHooks;
    type Auth = NoAuth;
    type AuthPolicy = OpenAuthPolicy;
    type ResourceType = Resources;
    const TYPE: Resources = Resources::Person;
}

impl SeaOrmResource for PersonResource {
    type Entity = person::Entity;
    type SeaOrmModel = person::Model;
    type ActiveModel = person::ActiveModel;
    type Column = person::Column;
    type PrimaryKey = <person::Entity as sea_orm::EntityTrait>::PrimaryKey;

    type ReadViewEntity = person::read_view::Entity;
    type ReadViewSeaOrmModel = person::read_view::Model;
    type ReadViewActiveModel = person::read_view::ActiveModel;
    type ReadViewColumn = person::read_view::Column;
    type ReadViewPrimaryKey = <person::read_view::Entity as sea_orm::EntityTrait>::PrimaryKey;

    fn model_field_to_column(field: &Self::ModelField) -> Self::Column {
        <person::ModelField as CrudColumns<person::Column>>::to_sea_orm_column(field)
    }

    fn read_model_field_to_column(field: &Self::ReadModelField) -> Self::ReadViewColumn {
        <person::read_view::ModelField as CrudColumns<person::read_view::Column>>::to_sea_orm_column(
            field,
        )
    }
}
