use crate::auth::{AuthRequirement, CrudAuthPolicy};
use crate::context::CrudContext;
use crate::prelude::{CrudResource, ResourceType};
use crudkit_core::collaboration::{CollabMessage, EntityCreated, EntityDeleted, EntityUpdated};
use crudkit_core::id::SerializableId;
use crudkit_core::validation::{FullSerializableValidations, PartialSerializableValidations};
use crudkit_wire_format::v1::CollabMessageV1;
use std::fmt::Debug;
use std::sync::Arc;

/// Delivers collaboration messages to the connected clients, e.g. over a websocket.
///
/// CrudKit sends validation status updates and entity change notifications through it. Messages
/// are handed over in their versioned wire form, ready to be serialized for clients.
pub trait CollaborationService {
    type Error: Debug + Send + Sync + 'static;

    /// Sends `message` to the connected clients that `audience` admits.
    ///
    /// The service must not send the message to a client that [`Audience::admits`] rejects,
    /// because the message names entities of a resource that the client may not read. The service
    /// may leave out more clients, e.g. those whose roles do not allow reading the resource.
    fn broadcast(
        &self,
        message: CollabMessageV1,
        audience: Audience,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

/// The clients that may receive a collaboration message: those that may read the resource that
/// the message is about.
///
/// It reflects the resource's [`CrudAuthPolicy::read_requirement`], which is all that CrudKit
/// knows about who may read a resource. Rules of the application, such as roles checked in
/// lifecycle hooks, are up to the [`CollaborationService`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Audience {
    /// The name of the resource that the message is about.
    pub resource_name: &'static str,
    /// What a client needs to read the resource.
    pub read_requirement: AuthRequirement,
}

impl Audience {
    /// The audience of messages about the resource `R`.
    #[must_use]
    pub fn of<R: CrudResource>() -> Self {
        Self {
            resource_name: R::TYPE.name(),
            read_requirement: R::AuthPolicy::read_requirement(),
        }
    }

    /// Whether a client may receive the message. `authenticated` tells whether the client
    /// presented the authentication that the resource's operations accept.
    #[must_use]
    pub fn admits(&self, authenticated: bool) -> bool {
        match self.read_requirement {
            AuthRequirement::None => true,
            AuthRequirement::Authenticated => authenticated,
        }
    }
}

pub(crate) async fn broadcast_creation_event<R: CrudResource>(
    context: &CrudContext<R>,
    serializable_id: SerializableId,
    with_validation_errors: bool,
) {
    if let Err(err) = context
        .collab_service
        .broadcast(
            CollabMessageV1::from(CollabMessage::EntityCreated(EntityCreated {
                resource_name: R::TYPE.name().to_owned(),
                entity_id: serializable_id,
                with_validation_errors,
            })),
            Audience::of::<R>(),
        )
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
        .broadcast(
            CollabMessageV1::from(CollabMessage::EntityUpdated(EntityUpdated {
                resource_name: R::TYPE.name().to_owned(),
                entity_id: serializable_id,
                with_validation_errors: has_violations,
            })),
            Audience::of::<R>(),
        )
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
        .broadcast(
            CollabMessageV1::from(CollabMessage::EntityDeleted(EntityDeleted {
                resource_name: R::TYPE.name().to_owned(),
                entity_id: serializable_id,
            })),
            Audience::of::<R>(),
        )
        .await
    {
        tracing::warn!("Failed to broadcast EntityDeleted event: {e:?}");
    }
}

pub(crate) async fn broadcast_partial_validation_result<R: CrudResource>(
    context: &CrudContext<R>,
    partial: PartialSerializableValidations,
) {
    if let Err(err) = context
        .collab_service
        .broadcast(
            CollabMessageV1::from(CollabMessage::PartialValidationResult(partial)),
            Audience::of::<R>(),
        )
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
        .broadcast(
            CollabMessageV1::from(CollabMessage::FullValidationResult(full)),
            Audience::of::<R>(),
        )
        .await
    {
        tracing::warn!("Failed to broadcast full validation result: {err:?}");
    }
}
