#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Storage-agnostic CRUD framework for Rust.
//!
//! This crate provides the core abstractions for building CRUD applications.
//! It is designed to be independent of any specific storage backend - adapters
//! like `crudkit-sea-orm` provide concrete implementations for specific ORMs.
//!
//! # Key Concepts
//!
//! - **CrudResource**: The central trait defining a CRUD resource
//! - **Repository**: Storage-agnostic data access trait
//! - **CrudLifetime**: Lifecycle hooks for CRUD operations
//! - **EntityValidator**: Validation framework for entities
//!
//! # Example
//!
//! ```ignore
//! #[derive(Debug)]
//! struct Article;
//!
//! impl CrudResource for Article {
//!     type ReadModel = ArticleReadModel;
//!     type CreateModel = ArticleCreateModel;
//!     type UpdateModel = ArticleUpdateModel;
//!     type Model = ArticleModel;
//!     // ... other associated types
//! }
//! ```

pub mod auth;
pub mod axum_routes;
pub mod collaboration;
pub mod context;
pub mod create;
pub mod data;
pub mod delete;
pub mod error;
pub mod lifetime;
pub mod read;
pub mod repository;
pub mod resource;
pub mod update;
pub mod validate;
pub mod validation;
pub mod validator;

// Re-export common modules for convenience.
pub use crudkit_core;
pub use crudkit_core::collaboration as crudkit_collaboration;
pub use crudkit_core::condition as crudkit_condition;
pub use crudkit_core::id as crudkit_id;
pub use crudkit_core::resource as crudkit_resource;
pub use crudkit_core::validation as crudkit_validation;

pub mod prelude {
    pub use crudkit_core;
    pub use crudkit_core::collaboration as crudkit_collaboration;
    pub use crudkit_core::condition as crudkit_condition;
    pub use crudkit_core::id as crudkit_id;
    pub use crudkit_core::resource as crudkit_resource;
    pub use crudkit_core::validation as crudkit_validation;

    // Derive macros that remain in crudkit-rs (storage-agnostic).
    pub use crudkit_rs_macros::CkResourceContext;
    pub use crudkit_core_macros::CkId;

    // Authentication and authorization.
    pub use super::auth::Auth;
    pub use super::auth::AuthExtractor;
    pub use super::auth::AuthRequirement;
    pub use super::auth::CrudAuthPolicy;
    pub use super::auth::DefaultAuthPolicy;
    pub use super::auth::NoAuth;
    pub use super::auth::OpenAuthPolicy;
    pub use super::auth::RequestContext;
    pub use super::auth::RequiresAuth;
    pub use super::auth::RestrictedAuthPolicy;

    // Errors.
    pub use super::axum_routes::AxumCrudError;
    pub use super::error::CrudError;

    // Core types.
    pub use super::collaboration::CollaborationService;
    pub use super::context::CrudContext;
    pub use super::resource::CrudResource;
    pub use super::resource::CrudResourceContext;
    pub use super::resource::ResourceType;

    // Data traits.
    pub use super::data::ConditionValueConverter;
    pub use super::data::CrudIdTrait; // Backward compatibility alias for HasId.
    pub use super::data::CrudModel; // Backward compatibility alias for Model.
    pub use super::data::Field;
    pub use super::data::FieldLookup;
    pub use super::data::FieldTrait; // Backward compatibility alias for Field.
    pub use super::data::HasId;
    pub use super::data::Model;

    // Lifetime hooks and related types.
    pub use super::lifetime::CrudLifetime;
    pub use super::lifetime::DeleteOperation;
    pub use super::lifetime::DeleteRequest;
    pub use super::lifetime::HookError;
    pub use super::lifetime::NoopError;
    pub use super::lifetime::NoopLifetimeHooks;
    pub use super::lifetime::ReadOperation;
    pub use super::lifetime::ReadRequest;
    pub use super::lifetime::ReadResult;
    pub use super::lifetime::UpdateRequest;

    // Repository.
    pub use super::repository::DeleteResult;
    pub use super::repository::NoopValidationResultRepository;
    pub use super::repository::Repository;
    pub use super::repository::RepositoryError;
    pub use super::repository::ValidationResultRepository;

    // Validation.
    pub use super::validation::CrudAction;
    pub use super::validation::ValidationContext;
    pub use super::validation::ValidationTrigger;
    pub use super::validation::When;
    pub use super::validator::AggregateValidator;
    pub use super::validator::EntityValidator;

    pub use super::create::create_one;
    // CRUD operations.
    pub use super::create::CreateOne;
    pub use super::delete::delete_by_id;
    pub use super::delete::delete_many;
    pub use super::delete::delete_one;
    pub use super::delete::DeleteById;
    pub use super::delete::DeleteMany;
    pub use super::delete::DeleteOne;
    pub use super::read::read_count;
    pub use super::read::read_many;
    pub use super::read::read_one;
    pub use super::read::ReadCount;
    pub use super::read::ReadMany;
    pub use super::read::ReadOne;
    pub use super::update::update_one;
    pub use super::update::UpdateOne;
}
