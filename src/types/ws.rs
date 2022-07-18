use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TypedMessage<T> {
    // TODO: Use an enum instead?
    pub message_type: String,
    pub data: T,
}
