use utoipa::ToSchema;

#[derive(Debug, ToSchema)]
pub enum CrudError {
    UnknownColumnSpecified(String),
    UnableToParseValueAsColType(String, String),
    DbError(String),
    ReadOneFoundNone,
}
