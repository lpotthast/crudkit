//! Collaboration service used by every resource of this example.

use crudkit_rs::crudkit_wire_format::v1::CollabMessageV1;
use crudkit_rs::prelude::CollaborationService;
use std::convert::Infallible;

/// Discards all collaboration messages.
///
/// A real application would broadcast these messages to connected clients, e.g. over a websocket.
#[derive(Debug, Default)]
pub struct NoopCollaborationService;

impl CollaborationService for NoopCollaborationService {
    type Error = Infallible;

    async fn broadcast(&self, message: CollabMessageV1) -> Result<(), Self::Error> {
        tracing::debug!(?message, "Discarding collaboration message.");
        Ok(())
    }
}
