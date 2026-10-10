//! Loading state of server-provided data.

use crate::http::RequestError;

/// State of data requested from a server.
#[derive(Debug, Clone, PartialEq)]
pub enum LoadState<T> {
    /// The request has not completed yet.
    Loading,
    /// The request returned data.
    Loaded(T),
    /// The request succeeded, but the requested entity does not exist.
    NotFound,
    /// The request failed.
    Failed(RequestError),
}

impl<T> LoadState<T> {
    /// Returns the loaded data, if any.
    pub fn loaded(&self) -> Option<&T> {
        match self {
            Self::Loaded(data) => Some(data),
            Self::Loading | Self::NotFound | Self::Failed(_) => None,
        }
    }

    /// Consumes the state and returns the loaded data, if any.
    pub fn into_loaded(self) -> Option<T> {
        match self {
            Self::Loaded(data) => Some(data),
            Self::Loading | Self::NotFound | Self::Failed(_) => None,
        }
    }

    /// Returns whether the request has not completed yet.
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    /// Maps loaded data.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> LoadState<U> {
        match self {
            Self::Loading => LoadState::Loading,
            Self::Loaded(data) => LoadState::Loaded(f(data)),
            Self::NotFound => LoadState::NotFound,
            Self::Failed(error) => LoadState::Failed(error),
        }
    }

    /// Converts a completed optional request result. `None` results become [`Self::NotFound`].
    pub fn from_optional_result(result: Result<Option<T>, RequestError>) -> Self {
        match result {
            Ok(Some(data)) => Self::Loaded(data),
            Ok(None) => Self::NotFound,
            Err(error) => Self::Failed(error),
        }
    }
}

impl<T> From<Result<T, RequestError>> for LoadState<T> {
    fn from(result: Result<T, RequestError>) -> Self {
        match result {
            Ok(data) => Self::Loaded(data),
            Err(error) => Self::Failed(error),
        }
    }
}
