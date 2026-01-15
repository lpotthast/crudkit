use crate::repository::RepositoryError;
use crudkit_condition::IntoAllEqualConditionError;
use snafu::{Backtrace, Snafu};
use std::sync::Arc;

// TODO: We should ensure that the error printed and/or serialized down to any frontend does not disclose secret information!
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum CrudError {
    #[snafu(display("CrudError: Repository error.\n\nCaused by:\n{reason:?}"))]
    Repository {
        reason: Arc<dyn RepositoryError>,
        backtrace: Backtrace,
    },

    #[snafu(display("CrudError: Could not convert to condition."))]
    IntoCondition {
        source: IntoAllEqualConditionError,
        backtrace: Backtrace,
    },

    #[snafu(display("CrudError: Entity not found."))]
    ReadOneFoundNone { backtrace: Backtrace },

    #[snafu(display("CrudError: Could not save validations.\n\nCaused by:\n{reason:?}"))]
    SaveValidations {
        reason: Arc<dyn RepositoryError>,
        backtrace: Backtrace,
    },

    #[snafu(display("CrudError: Could not delete validations.\n\nCaused by:\n{reason:?}"))]
    DeleteValidations {
        reason: Arc<dyn RepositoryError>,
        backtrace: Backtrace,
    },

    // TODO: We do not have a variant `CreateAborted`. Why should we handle these cases differently?
    #[snafu(display("CrudError: Read operation aborted. Reason: {reason}"))]
    ReadAborted {
        reason: String,
        backtrace: Backtrace,
    },
}
