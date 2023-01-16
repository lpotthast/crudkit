use snafu::{Backtrace, Snafu};
use utoipa::ToSchema;

#[derive(Debug, Snafu, ToSchema)]
#[snafu(visibility(pub))]
pub enum CrudError {
    #[snafu(display("CrudError: Column '{column_name}' not found.\n\nBacktrace:\n{backtrace}"))]
    UnknownColumnSpecified {
        column_name: String,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "CrudError: Unable to parse value for column'{column_name}' to column type: '{reason}'\n\nBacktrace:\n{backtrace}"
    ))]
    UnableToParseValueAsColType {
        column_name: String,
        reason: String,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "CrudError: Database error.\n\nCaused by:\n{reason}\n\nBacktrace:\n{backtrace}"
    ))]
    Db {
        reason: String,
        backtrace: Backtrace,
    }, // TODO: Change reason to:  source: DbErr,

    #[snafu(display("CrudError: Entity not found.\n\nBacktrace:\n{backtrace}"))]
    ReadOneFoundNone { backtrace: Backtrace },

    #[snafu(display("CrudError: Could not save validations.\n\nBacktrace:\n{backtrace}"))]
    SaveValidations { backtrace: Backtrace },
}
