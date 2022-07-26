use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TypedMessage<T> {
    // TODO: Use an enum instead?
    pub message_type: String,
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct PeakTypedMessage<M> {
    pub message_type: M,
    pub data: Box<serde_json::value::RawValue>,
}
