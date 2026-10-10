//! HTTP transport of the generated CRUD routes: request execution, endpoints, and errors.

mod endpoint;
mod error;
mod executor;
mod request;

pub(crate) use endpoint::{CrudEndpoint, CrudOperation};
pub use error::RequestError;
pub use executor::{NewClientPerRequestExecutor, ReqwestExecutor};
