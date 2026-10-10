use crudkit_core::validation::PartialSerializableAggregateViolations;
use crudkit_wire_format::v1::{ErrorResponseV1, ErrorV1};
use thiserror::Error as ThisError;

/// Failure of a request to the generated CRUD routes.
///
/// The server's failures arrive as version 1 error bodies and are classified by their `kind`. A
/// response without such a body, e.g. from a proxy, is classified by its HTTP status instead.
///
/// | Variant | Cause |
/// |---------|-------|
/// | `BadRequest` | The server rejected the request as malformed (400). |
/// | `Unauthorized` | Authentication is required (401). |
/// | `Forbidden` | The caller lacks permission (403). |
/// | `NotFound` | The selected entity does not exist (404). |
/// | `UnprocessableEntity` | Business rules rejected the operation (422). |
/// | `CriticalValidationErrors` | Critical validation errors prevent the operation (422). |
/// | `InternalServerError` | The server failed (500). |
/// | `InvalidRequest` | The client could not build the request; nothing was sent. |
/// | `Request` | The request failed in transport or the server responded with another status. |
/// | `Deserialize` | The server's success response could not be deserialized. |
///
/// # Example
///
/// ```ignore
/// on_save_failed=move |error| match error {
///     RequestError::Forbidden(reason) => show_toast("Permission Denied", reason),
///     RequestError::UnprocessableEntity(reason) => show_toast("Cannot Proceed", reason),
///     error => show_toast("Error", error.to_string()),
/// }
/// ```
// TODO: All our other libraries use `snafu` for error handling. We should probably switch to that and remove the thiserror dependency.
#[derive(ThisError, Clone, Debug, PartialEq)]
pub enum RequestError {
    /// The server rejected the request as malformed (HTTP 400).
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// The operation requires authentication (HTTP 401).
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// The user lacks permission to perform the operation (HTTP 403).
    #[error("Permission denied: {0}")]
    Forbidden(String),

    /// The selected entity does not exist (HTTP 404).
    #[error("Not found: {0}")]
    NotFound(String),

    /// Business rules rejected the operation (HTTP 422).
    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    /// Critical validation errors prevent the operation (HTTP 422).
    #[error("Validation failed: {message}")]
    CriticalValidationErrors {
        message: String,
        /// The violations that prevent the operation.
        violations: PartialSerializableAggregateViolations,
    },

    /// An internal server error occurred (HTTP 500).
    #[error("Server error: {0}")]
    InternalServerError(String),

    /// The client could not build the request, e.g. because a URL, body, or condition is invalid.
    /// Nothing was sent.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// The request failed in transport, or the server responded with an unexpected status.
    #[error("Network error: {0}")]
    Request(String),

    /// The body of a success response could not be deserialized.
    #[error("Invalid response: {0}")]
    Deserialize(String),
}

/// Maps a non-success HTTP response to the matching `RequestError` variant.
pub(crate) async fn error_response_to_request_error(response: reqwest::Response) -> RequestError {
    let status = response.status().as_u16();
    match response.bytes().await {
        Ok(body) => request_error_from_body(status, &body),
        Err(err) => RequestError::Request(err.to_string()),
    }
}

/// Classifies a failed response by its version 1 error body, or by its status if it has none.
fn request_error_from_body(status: u16, body: &[u8]) -> RequestError {
    if let Ok(response) = serde_json::from_slice::<ErrorResponseV1>(body) {
        return match response.error {
            ErrorV1::BadRequest { message } => RequestError::BadRequest(message),
            ErrorV1::Unauthorized { message } => RequestError::Unauthorized(message),
            ErrorV1::Forbidden { message } => RequestError::Forbidden(message),
            ErrorV1::NotFound { message } => RequestError::NotFound(message),
            ErrorV1::UnprocessableEntity { message } => RequestError::UnprocessableEntity(message),
            ErrorV1::CriticalValidationErrors {
                message,
                violations,
            } => RequestError::CriticalValidationErrors {
                message,
                violations: violations.into(),
            },
            ErrorV1::InternalServerError { message } => RequestError::InternalServerError(message),
        };
    }
    let text = String::from_utf8_lossy(body).into_owned();
    match status {
        400 => RequestError::BadRequest(text),
        401 => RequestError::Unauthorized(text),
        403 => RequestError::Forbidden(text),
        404 => RequestError::NotFound(text),
        422 => RequestError::UnprocessableEntity(text),
        500 => RequestError::InternalServerError(text),
        code => RequestError::Request(format!("Code: {code}, Text: {text}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;
    use crudkit_core::validation::violation::Violation;

    fn v1_body(error: &serde_json::Value) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({ "wire_format_version": 1, "error": error }))
            .expect("body should serialize")
    }

    #[test]
    fn version_1_error_bodies_are_classified_by_kind_not_status() {
        let cases = [
            ("bad_request", RequestError::BadRequest("m".to_owned())),
            ("unauthorized", RequestError::Unauthorized("m".to_owned())),
            ("forbidden", RequestError::Forbidden("m".to_owned())),
            ("not_found", RequestError::NotFound("m".to_owned())),
            (
                "unprocessable_entity",
                RequestError::UnprocessableEntity("m".to_owned()),
            ),
            (
                "internal_server_error",
                RequestError::InternalServerError("m".to_owned()),
            ),
        ];
        for (kind, expected) in cases {
            // The status deliberately disagrees, to show that the body decides.
            let body = v1_body(&serde_json::json!({ "kind": kind, "message": "m" }));
            assert_that!(request_error_from_body(418, &body)).is_equal_to(expected);
        }
    }

    #[test]
    fn critical_validation_errors_keep_their_violations() {
        let body = v1_body(&serde_json::json!({
            "kind": "critical_validation_errors",
            "message": "Critical validation errors prevent the operation.",
            "violations": {
                "general": [{ "severity": "critical", "message": "Too many Seekers." }],
                "create": null,
                "by_entity": [],
            },
        }));

        let RequestError::CriticalValidationErrors { violations, .. } =
            request_error_from_body(422, &body)
        else {
            panic!("expected critical validation errors");
        };
        assert_that!(violations.general.map(|violations| violations.violations))
            .is_equal_to(Some(vec![Violation::critical("Too many Seekers.")]));
    }

    #[test]
    fn other_bodies_are_classified_by_status() {
        assert_that!(request_error_from_body(403, b"nope"))
            .is_equal_to(RequestError::Forbidden("nope".to_owned()));
        assert_that!(request_error_from_body(502, b"gateway"))
            .is_equal_to(RequestError::Request("Code: 502, Text: gateway".to_owned()));
    }
}
