//! What applications declare: instance configuration, actions, field renderers, view registries,
//! and texts.
//!
//! Configuration is plain data. Mounting it creates an [`crate::instance`]; CrudKit's hooks and
//! atoms read it from there.

mod actions;
mod instance;
mod renderers;
mod texts;
mod views;

pub use actions::*;
pub use instance::*;
pub use renderers::*;
pub use texts::*;
pub use views::*;
