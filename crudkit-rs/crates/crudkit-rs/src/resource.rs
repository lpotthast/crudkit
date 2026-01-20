use crate::{
    auth::{AuthExtractor, CrudAuthPolicy},
    lifetime::CrudLifetime,
    prelude::*,
    GetIdFromModel,
};

use crudkit_id::Id;

use crate::repository::ValidationResultRepository;
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

    /// The type representing the "primary key" or "ID" of the update model.
    ///
    /// We only care about this models ID, as both the create and read models are not persistable
    /// on their own and must never be identified.
    type Id: Id + Clone;

    type Repository: Repository<Self>;

    // The service with which validation results can be managed: read, stored, ...
    type ValidationResultRepository: ValidationResultRepository;
    // TODO: Remove these old generics and bounds. ValidationResultSaver is no longer generic, because we want to support the unified table approach, where one repo cannot be bound to one specific id type.
    //  In case of individual tables, and individual repos, such a repo might be generic over the ID, but this trait no longer enforces that it is over the ID defined here.
    // <
    //             <Self::CrudColumn as CrudColumns<Self::Column, Self::Model, Self::ActiveModel>>::Id,
    //         > + 'static

    /// A type that can be used for collaboration purposes (sharing information with other users
    /// of the system).
    ///
    /// This could be implemented using WebSockets for example.
    type CollaborationService: CollaborationService + 'static;

    /// An instance of this type is made available in all lifetime operations.
    /// Use this to supply arbitrary data, like custom services.
    type Context: CrudResourceContext + Send + Sync + 'static;

    /// This type is `Default` created at the start of any operation (create, read, update, delete)
    /// and passed mutably to all lifecycle hooks called in that operation. For example, when
    /// creating an entity, the same instance of this type is passed to `before_create` and
    /// `after_create`.
    ///
    /// In case that an operation triggers multiple lifecycle hooks (like create), this type can
    /// allow you to hold onto some data across these hooks for later reference.
    ///
    /// Simply set this to `()` if not required.
    type HookData: Default + Send + Sync + 'static;

    type Lifetime: CrudLifetime<Self>;

    /// Authentication type for this resource.
    ///
    /// Must implement [`AuthExtractor`](AuthExtractor) for Axum route generation.
    /// Use [`NoAuth`](NoAuth) for public resources.
    type Auth: AuthExtractor;

    /// Per-operation authorization policy.
    ///
    /// Defines which operations require authentication.
    /// Use [`DefaultAuthPolicy`](DefaultAuthPolicy) for standard behavior
    /// (public reads, authenticated writes).
    type AuthPolicy: CrudAuthPolicy;

    type ResourceType: ResourceType;
    const TYPE: Self::ResourceType;
}

pub trait ResourceType: Debug + Clone + Copy + PartialEq + Eq {
    fn name(&self) -> &'static str;
}
