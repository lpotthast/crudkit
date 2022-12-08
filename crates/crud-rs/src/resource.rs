use crate::{lifetime::CrudLifetime, prelude::*, GetIdFromModel};
use crud_shared_types::prelude::Id;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, EntityTrait, FromQueryResult,
    IntoActiveModel, ModelTrait,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, hash::Hash};

pub trait CrudResource: Sized {
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

    type Column: ColumnTrait + Clone + Send + Sync + 'static;

    type CrudColumn: CrudColumns<Self::Column, Self::Model, Self::ActiveModel, Id = Self::Id>
        + Eq
        + Hash
        + DeserializeOwned
        + Clone
        + Send
        + Sync
        + 'static;

    type Id: Id + Clone;

    type Validator: EntityValidatorsTrait<Self>;

    // The service with which validation results can be managed: read, stored, ...
    type ValidationResultRepository: ValidationResultSaverTrait<<Self::CrudColumn as CrudColumns<Self::Column, Self::Model, Self::ActiveModel>>::Id>;

    type Context: CrudResourceContext + Send + Sync + 'static;

    type HookData: Default + Send + Sync + 'static;
    type Lifetime: CrudLifetime<Self>;

    type ResourceType: Debug + Into<&'static str> + Clone + Copy;
    const TYPE: Self::ResourceType;
}

trait ResourceLifecycle {
    fn before_create();
}
