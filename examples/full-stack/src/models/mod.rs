//! Frontend models shared by the server-side renderer and the hydrated WASM client.
//!
//! These mirror the wire format of the backend resources defined in `crate::server`.

pub mod club;
pub mod person;

/// Returns the current UTC date and time without an offset.
pub(crate) fn now_primitive() -> time::PrimitiveDateTime {
    let now = time::OffsetDateTime::now_utc();
    time::PrimitiveDateTime::new(now.date(), now.time())
}
