//! Collaboration service used by every resource of this example.

use crudkit_rs::crudkit_wire_format::v1::{CollabEventV1, CollabMessageV1};
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
        // The message can carry validation findings, which must not reach the logs.
        let kind = match message.event {
            CollabEventV1::EntityCreated { .. } => "entity_created",
            CollabEventV1::EntityUpdated { .. } => "entity_updated",
            CollabEventV1::EntityDeleted { .. } => "entity_deleted",
            CollabEventV1::PartialValidationResult { .. } => "partial_validation_result",
            CollabEventV1::FullValidationResult { .. } => "full_validation_result",
        };
        tracing::debug!(kind, "Discarding collaboration message.");
        Ok(())
    }
}
