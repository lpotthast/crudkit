//! Validation result persistence for crudkit.
//!
//! This module provides a unified validation repository that supports two storage modes:
//!
//! - **Unified mode**: All validation results for all resources are stored in a single
//!   `CrudkitValidation` table, with a `resource_type` column to distinguish them.
//!
//! - **Per-resource mode**: Each resource has its own validation table (e.g., `UserValidation`,
//!   `ClubValidation`), matching the traditional per-entity-type storage pattern.
//!
//! The entity ID is stored as JSON, which allows for both simple IDs (e.g., `123`) and
//! composite keys (e.g., `{"user_id":1,"event_id":2}`).

pub mod model;
pub mod repository;
