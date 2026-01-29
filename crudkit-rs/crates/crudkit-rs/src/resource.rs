//! Core resource trait defining the shape of CRUD resources.
//!
//! This module is storage-agnostic - it defines resources in terms of
//! ReadModel, CreateModel, and UpdateModel without coupling to any
//! specific storage backend like SeaORM.

use crate::{
    auth::{AuthExtractor, CrudAuthPolicy},
    data::{ConditionValueConverter, CrudIdTrait, CrudModel, FieldLookup, FieldTrait},
    lifetime::CrudLifetime,
    prelude::*,
};

use crudkit_id::Id;

use crate::data::CreateModelTrait;
use crate::repository::ValidationResultRepository;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::hash::Hash;

/// Marker trait for resource-specific context types.
///
/// The `CrudContext` (to be instantiated by you once for each resource you define), stores a value
/// of this type. It is later passed to operations specific to this resource, e.g. lifecycle hooks,
/// allowing you to access custom application state during their execution.
///
/// # Example
///
/// When no custom data must be captured, just define a unit struct.
/// You can always derive the `CrudResourceContext` impl using the `CkResourceContext` derive macro.
///
/// ```
/// use crudkit_rs::prelude::*;
///
/// #[derive(Debug, CkResourceContext)]
/// pub struct ArticleResourceContext;
/// ```
pub trait CrudResourceContext {}

/// Central trait defining a CRUD resource.
///
/// This trait is storage-agnostic and defines resources in terms of:
/// - **ReadModel**: The model returned from read queries (may come from a SQL view)
/// - **CreateModel**: The DTO used to create new entities
/// - **UpdateModel**: The DTO used to update existing entities
///
/// Each model has an associated Field type for typed field access.
///
/// # Design Philosophy
///
/// This design aligns with `CrudMainTrait` from crudkit-web, where resources
/// are defined in terms of models and fields rather than ORM-specific types.
/// The storage adapter (e.g., crudkit-sea-orm) handles the conversion between
/// these abstract models and the underlying storage mechanism.
pub trait CrudResource: Sized + Debug {
    // =========================================================================
    // Read Model (for querying entities).
    // =========================================================================

    /// The model returned from read operations.
    ///
    /// This might be backed by a SQL view rather than a direct table.
    /// Must be identifiable and serializable for API responses.
    type ReadModel: CrudModel<Field = Self::ReadModelField>
        + CrudIdTrait<Id = Self::ReadModelId>
        + Serialize
        + Clone
        + Send
        + Sync
        + 'static;

    /// The ID type for read models.
    type ReadModelId: Id + Clone + Send + Sync + 'static;

    /// The field enum for read models, used for ordering and filtering.
    type ReadModelField: FieldTrait
        + FieldLookup
        + ConditionValueConverter
        + DeserializeOwned
        + Eq
        + Hash
        + Clone
        + Send
        + Sync
        + 'static;

    // =========================================================================
    // Create Model (for creating new entities).
    // =========================================================================

    /// The DTO used to create new entities.
    ///
    /// Typically doesn't have an ID if the ID is generated during insertion.
    type CreateModel: CrudModel<Field = Self::CreateModelField>
        + CreateModelTrait
        + DeserializeOwned
        + Debug
        + Clone
        + Send
        + Sync
        + 'static;

    /// Type representing individual fields of the `CreateModel`.
    type CreateModelField: FieldTrait + Clone + Send + Sync + 'static;

    // =========================================================================
    // Update Model (for updating existing entities).
    // =========================================================================

    /// The DTO used to update existing entities.
    ///
    /// Contains the data to update. The entity to update is identified
    /// via a condition in the request.
    type UpdateModel: CrudModel<Field = Self::UpdateModelField>
        + DeserializeOwned
        + Debug
        + Clone
        + Send
        + Sync
        + 'static;

    /// Type representing individual fields of the `UpdateModel`.
    type UpdateModelField: FieldTrait + Clone + Send + Sync + 'static;

    // =========================================================================
    // Entity Model - the actual persisted entity
    // =========================================================================

    /// The model representing the actual persisted entity.
    ///
    /// This is the "real" entity that gets stored. It's used in lifecycle hooks
    /// for validation and in the repository for fetch/delete operations.
    /// Unlike ReadModel (which may be a view), this represents the actual table.
    type Model: CrudModel<Field = Self::ModelField>
        + CrudIdTrait<Id = Self::Id>
        + Serialize
        + Clone
        + Send
        + Sync
        + 'static;

    /// The ID type for the persisted entity.
    type Id: Id + Clone + Send + Sync + 'static;

    /// The field enum for the persisted entity model.
    type ModelField: FieldTrait
        + FieldLookup
        + ConditionValueConverter
        + DeserializeOwned
        + Eq
        + Hash
        + Clone
        + Send
        + Sync
        + 'static;

    // =========================================================================
    // Infrastructure types.
    // =========================================================================

    /// The repository implementation for this resource.
    type Repository: Repository<Self>;

    /// The service for persisting and retrieving validation results.
    type ValidationResultRepository: ValidationResultRepository;

    /// Service for collaboration (e.g., WebSocket broadcasting).
    type CollaborationService: CollaborationService + 'static;

    /// Resource-specific context made available in lifecycle operations.
    type Context: CrudResourceContext + Send + Sync + 'static;

    /// Data passed through lifecycle hooks within a single operation.
    ///
    /// `Default` created at the start of any operation (create, read, update, delete)
    /// and passed mutably to all lifecycle hooks in that operation.
    ///
    /// Set to `()` if not needed.
    type HookData: Default + Send + Sync + 'static;

    /// Lifecycle hooks implementation for this resource.
    type Lifetime: CrudLifetime<Self>;

    /// Authentication type for this resource.
    ///
    /// Must implement [`AuthExtractor`] for Axum route generation.
    /// Use `NoAuth` for public resources.
    type Auth: AuthExtractor;

    /// Per-operation authorization policy.
    ///
    /// Defines which operations require authentication.
    type AuthPolicy: CrudAuthPolicy;

    /// The resource type identifier.
    type ResourceType: ResourceType;

    /// The constant identifying this resource type.
    const TYPE: Self::ResourceType;
}

/// Trait for resource type identifiers.
///
/// Resource types provide a static name used for routing, validation storage, etc.
pub trait ResourceType: Debug + Clone + Copy + PartialEq + Eq {
    /// Returns the resource name as a static string.
    fn name(&self) -> &'static str;
}
