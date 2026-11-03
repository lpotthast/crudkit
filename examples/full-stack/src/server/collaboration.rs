//! Collaboration service used by every resource of this example.

use crudkit_rs::crudkit_wire_format::v1::CollabMessageV1;
use crudkit_rs::prelude::{Audience, CollaborationService};
use std::convert::Infallible;

/// Discards all collaboration messages.
///
/// A real application would send these messages to the connected clients that the audience admits,
/// e.g. over a websocket.
#[derive(Debug, Default)]
pub struct NoopCollaborationService;

impl CollaborationService for NoopCollaborationService {
    type Error = Infallible;

    async fn broadcast(
        &self,
        message: CollabMessageV1,
        _audience: Audience,
    ) -> Result<(), Self::Error> {
        tracing::debug!(?message, "Discarding collaboration message.");
        Ok(())
    }
}
