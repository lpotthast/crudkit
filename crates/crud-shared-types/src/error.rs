

#[derive(Debug)]
pub enum CrudError {
    UnknownColumnSpecified(String),
    UnableToParseValueAsColType(String, String),
    UnableToParseAsEntity(String, String),
    DbError(String),
    ReadOneFoundNone,
}
