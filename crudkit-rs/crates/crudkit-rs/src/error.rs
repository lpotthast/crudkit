use crate::repository::RepositoryError;
use snafu::{Backtrace, Snafu};
use std::sync::Arc;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum CrudError {
    #[snafu(display("CrudError: Repository error.\n\nCaused by:\n{reason:?}"))]
    Repository {
        reason: Arc<dyn RepositoryError>,
        backtrace: Backtrace,
    },

    #[snafu(display("CrudError: Entity not found."))]
    ReadOneFoundNone { backtrace: Backtrace },

    #[snafu(display("CrudError: Could not save validations.\n\nCaused by:\n{reason:?}"))]
    SaveValidations {
        reason: Arc<dyn RepositoryError>, // Use ValidationRepositoryError!
        backtrace: Backtrace,
    },
    #[snafu(display("CrudError: Could not delete validations.\n\nCaused by:\n{reason:?}"))]
    DeleteValidations {
        reason: Arc<dyn RepositoryError>, // Use ValidationRepositoryError!
        backtrace: Backtrace,
    },
}
