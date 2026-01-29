//! Validation context types describing when and why validation runs.

use serde::{Deserialize, Serialize};

/// The CRUD action that triggered validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrudAction {
    Create,
    Read,
    Update,
    Delete,
}

/// Whether validation occurs before or after the CRUD action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum When {
    Before,
    After,
}

/// Context describing when a CRUD action triggered validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationContext {
    /// The CRUD action that lead to the validation.
    pub action: CrudAction,

    /// Whether the validation occurs before or after applying the CRUD action.
    /// Critical violations created before the action is applied will prevent its application.
    pub when: When,
}

/// What triggered the validation run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationTrigger {
    /// Validation triggered by a CRUD action.
    CrudAction(ValidationContext),

    /// Validation triggered by a global/aggregate validation run.
    GlobalValidation,
}
