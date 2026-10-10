use crate::context::CrudContext;
use crate::prelude::{CrudResource, ResourceType};
use crudkit_core::collaboration::{CollabMessage, EntityCreated, EntityDeleted, EntityUpdated};
use crudkit_core::id::SerializableId;
use crudkit_core::validation::{FullSerializableValidations, PartialSerializableValidations};
use crudkit_wire_format::v1::CollabMessageV1;
use std::fmt::Debug;
use std::sync::Arc;

/// We assume that users establish a websocket connection.
///
/// This trait allows crudkit to communicate with users through websocket messages.
///
/// It is used to send validation status updates and entity change notifications. Messages are
/// handed over in their versioned wire form, ready to be serialized for clients.
pub trait CollaborationService {
    type Error: Debug + Send + Sync + 'static;

    /// Send a message to all connected users.
    fn broadcast(
        &self,
        message: CollabMessageV1,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub(crate) async fn broadcast_creation_event<R: CrudResource>(
    context: &CrudContext<R>,
    serializable_id: SerializableId,
    with_validation_errors: bool,
) {
    if let Err(err) = context
        .collab_service
        .broadcast(CollabMessageV1::from(CollabMessage::EntityCreated(
            EntityCreated {
                resource_name: R::TYPE.name().to_owned(),
                entity_id: serializable_id,
                with_validation_errors,
            },
        )))
        .await
    {
        tracing::warn!("Failed to broadcast EntityCreated event: {err:?}");
    }
}

pub(crate) async fn broadcast_updated_event<R: CrudResource>(
    context: &Arc<CrudContext<R>>,
    serializable_id: SerializableId,
    has_violations: bool,
) {
    if let Err(err) = context
        .collab_service
        .broadcast(CollabMessageV1::from(CollabMessage::EntityUpdated(
            EntityUpdated {
                resource_name: R::TYPE.name().to_owned(),
                entity_id: serializable_id,
                with_validation_errors: has_violations,
            },
        )))
        .await
    {
        tracing::error!("Failed to broadcast EntityUpdated event: {err:?}");
    }
}

pub(crate) async fn broadcast_deletion_event<R: CrudResource>(
    context: &Arc<CrudContext<R>>,
    serializable_id: SerializableId,
) {
    if let Err(e) = context
        .collab_service
        .broadcast(CollabMessageV1::from(CollabMessage::EntityDeleted(
            EntityDeleted {
                resource_name: R::TYPE.name().to_owned(),
                entity_id: serializable_id,
            },
        )))
        .await
    {
        tracing::warn!("Failed to broadcast EntityDeleted deleted: {e:?}");
    }
}

pub(crate) async fn broadcast_partial_validation_result<R: CrudResource>(
    context: &CrudContext<R>,
    partial: PartialSerializableValidations,
) {
    if let Err(err) = context
        .collab_service
        .broadcast(CollabMessageV1::from(
            CollabMessage::PartialValidationResult(partial),
        ))
        .await
    {
        tracing::warn!("Failed to broadcast partial validation result: {err:?}");
    }
}

#[allow(dead_code)]
pub(crate) async fn broadcast_full_validation_result<R: CrudResource>(
    context: &CrudContext<R>,
    full: FullSerializableValidations,
) {
    if let Err(err) = context
        .collab_service
        .broadcast(CollabMessageV1::from(CollabMessage::FullValidationResult(
            full,
        )))
        .await
    {
        tracing::warn!("Failed to broadcast full validation result: {err:?}");
    }
}
