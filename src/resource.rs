use crate::prelude::*;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, EntityTrait, FromQueryResult,
    IntoActiveModel, ModelTrait,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, hash::Hash};

pub trait CrudResource {
    type Entity: EntityTrait<Model = Self::Model, Column = Self::Column>
        + MaybeColumnTrait
        + Clone
        + Send
        + Sync
        + 'static;
    type CreateModel: CreateModelTrait
        + DeserializeOwned
        + Into<Self::Model>
        + Debug
        + Clone
        + Send
        + Sync
        + 'static;
    type Model: ModelTrait<Entity = Self::Entity>
        + IntoActiveModel<Self::ActiveModel>
        + FromQueryResult
        + Serialize
        + Clone
        + Send
        + Sync
        + 'static;
    type ActiveModel: ActiveModelTrait<Entity = Self::Entity>
        + ActiveModelBehavior
        // TODO: just use Self::Column here?
        + ExcludingColumnsOnInsert<Self::Column>
        + From<Self::Model>
        + UpdateActiveModelTrait<Self::CreateModel>
        + Clone
        + Send
        + Sync
        + 'static;
    type Column: ColumnTrait + Clone + Send + Sync + 'static;
    type CrudColumn: CrudColumns<Self::Column>
        + Eq
        + Hash
        + DeserializeOwned
        + Clone
        + Send
        + Sync
        + 'static;
    type Validator:
        EntityValidatorsTrait<Self::ActiveModel>;
    type ValidationResultRepository:
        ValidationResultSaverTrait;
    type ResourceType: Debug + Into<&'static str> + Clone + Copy;

    const TYPE: Self::ResourceType;
}

trait ResourceLifecycle {
    fn before_create();
}
