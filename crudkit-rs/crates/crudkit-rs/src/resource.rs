use crate::{lifetime::CrudLifetime, prelude::*, GetIdFromModel};

use crudkit_id::Id;

use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, EntityTrait, FromQueryResult,
    IntoActiveModel, ModelTrait,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, hash::Hash};

// TODO: Can this be a member of CrudContext?
/// Every crud resource needs its own resource context in which any data imaginable can be presented.
/// This context is used in some operations specific to this contexts resource.
pub trait CrudResourceContext {}

pub trait CrudResource: Sized + Debug {
    // The 'real' entity used when reading / querying entities. Might as well be a SQL view instead of a real table ;)
    type ReadViewEntity: EntityTrait<Model = Self::ReadViewModel, Column = Self::ReadViewColumn>
        + MaybeColumnTrait
        + Clone
        + Send
        + Sync
        + 'static;

    // The model of the read / queried data.
    type ReadViewModel: ModelTrait<Entity = Self::ReadViewEntity>
        // TODO: + GetIdFromModel<Id = ReadViewId>
        + IntoActiveModel<Self::ReadViewActiveModel>
        + FromQueryResult
        + Serialize
        + Clone
        + Send
        + Sync
        + 'static;

    // The active model of the read / queried data.
    type ReadViewActiveModel: ActiveModelTrait<Entity = Self::ReadViewEntity>
        + ActiveModelBehavior
        + From<Self::ReadViewModel>
        // Does not need ExcludingColumnsOnInsert<Self::ReadViewColumn>
        // Does not need UpdateActiveModelTrait<Self::ReadViewCreateModel>
        + Clone
        + Send
        + Sync
        + 'static;

    // The 'real' column type (an enum) of the read / queried data.
    type ReadViewColumn: ColumnTrait + Clone + Send + Sync + 'static;

    type ReadViewCrudColumn: CrudColumns<Self::ReadViewColumn, Self::ReadViewModel, Self::ReadViewActiveModel>
        + Debug
        + Eq
        + Hash
        + DeserializeOwned
        + Clone
        + Send
        + Sync
        + 'static;

    // The 'real' entity (sea-orm).
    type Entity: EntityTrait<Model = Self::Model, Column = Self::Column>
        + MaybeColumnTrait
        + Clone
        + Send
        + Sync
        + 'static;

    // Used to create en entity
    type CreateModel: CreateModelTrait<Self::ActiveModel>
        + DeserializeOwned
        + Debug
        + Clone
        + Send
        + Sync
        + 'static;

    // Used to update an entity
    type UpdateModel: UpdateModelTrait + DeserializeOwned + Debug + Clone + Send + Sync + 'static;

    type Model: ModelTrait<Entity = Self::Entity>
        + IntoActiveModel<Self::ActiveModel>
        + GetIdFromModel<Id = Self::Id>
        + FromQueryResult
        + Serialize
        + Clone
        + Send
        + Sync
        + 'static;

    type ActiveModel: ActiveModelTrait<Entity = Self::Entity>
        + ActiveModelBehavior
        + From<Self::Model>
        + UpdateActiveModelTrait<Self::UpdateModel>
        + Clone
        + Send
        + Sync
        + 'static;

    type Column: ColumnTrait + Debug + Clone + Send + Sync + 'static;

    type CrudColumn: CrudColumns<Self::Column, Self::Model, Self::ActiveModel, Id = Self::Id>
        + Debug
        + Eq
        + Hash
        + DeserializeOwned
        + Clone
        + Send
        + Sync
        + 'static;

    type Id: Id + Clone;

    type Repository: Repository<Self>;

    type Validator: EntityValidatorsTrait<Self>;

    // The service with which validation results can be managed: read, stored, ...
    type ValidationResultRepository: ValidationResultSaverTrait<
            <Self::CrudColumn as CrudColumns<Self::Column, Self::Model, Self::ActiveModel>>::Id,
        > + 'static;

    type WebsocketService: CrudWebsocketService + 'static;

    /// An instance of this type is made available in all lifetime operations.
    /// Use this to supply arbitrary data, like custom services.
    type Context: CrudResourceContext + Send + Sync + 'static;

    type HookData: Default + Send + Sync + 'static;
    type Lifetime: CrudLifetime<Self>;

    type ResourceType: Debug + Into<&'static str> + Clone + Copy;
    const TYPE: Self::ResourceType;
}
