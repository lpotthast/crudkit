//! The runtime of mounted instances: their context, manager, and navigation, and the structural
//! components that mount an instance and render its current view without markup of their own.

mod context;
mod manager;
mod navigation;
mod outlet;

pub use context::*;
pub use manager::*;
pub use navigation::*;
pub use outlet::*;
