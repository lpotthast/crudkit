#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

//! Versioned wire format of CrudKit's CRUD API.
//!
//! Every module describes one version of the serialized contract between clients and servers. Its
//! types are data transfer objects only: they carry no behavior, and both the backend and the
//! frontend convert them into their own internal representations at the transport boundary. A
//! breaking change adds a new version module instead of changing an existing one.

pub mod v1;
mod version;

pub use version::{DetectVersionError, VERSION_FIELD, WireFormatVersion};
