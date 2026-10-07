//! Version 1 of the wire format.
//!
//! Every type carries a `V1` suffix, so that mapping code can name it next to the internal type it
//! represents. Every request and response body carries a [`WireFormatVersionV1`] in its
//! `wire_format_version` field.
//!
//! Version 1 is released and frozen: it covers every serialized contract between clients and
//! servers, and its fixture tests pin its JSON. A breaking change adds a new version module.

mod collaboration;
mod condition;
mod error;
mod id;
mod order;
mod request;
mod response;
mod validation;
mod version;

pub use collaboration::{
    CollabEventV1, CollabMessageV1, ResourceFullViolationsV1, ResourcePartialViolationsV1,
};
pub use condition::{
    ConditionClauseV1, ConditionClauseValueV1, ConditionElementV1, ConditionV1, OperatorV1,
};
pub use error::{ErrorResponseV1, ErrorV1};
pub use id::{IdValueV1, SerializableIdEntryV1, SerializableIdV1};
pub use order::OrderV1;
pub use request::{
    CreateOneV1, DeleteByIdV1, DeleteManyV1, DeleteOneV1, ReadCountV1, ReadManyV1, ReadOneV1,
    UpdateOneV1,
};
pub use response::{
    DeletedManyV1, DeletedV1, FailedDeletionV1, ReadCountResponseV1, ReadManyResponseV1,
    ReadOneResponseV1, SavedV1,
};
pub use validation::{
    EntityViolationsV1, FullViolationsV1, PartialViolationsV1, SeverityV1, ViolationV1,
};
pub use version::WireFormatVersionV1;
