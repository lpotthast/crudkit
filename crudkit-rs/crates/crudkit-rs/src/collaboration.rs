use crudkit_collaboration::CollabMessage;
use std::fmt::Debug;

/// We assume that users establish a websocket connection.
///
/// This trait allows crudkit to communicate with users through websocket messages.
///
/// It is used to send validation status updates and entity change notifications.
pub trait CollaborationService {
    type Error: Debug + Send + Sync + 'static;

    /// Send a message to all connected users.
    fn broadcast_json(
        &self,
        json: CollabMessage,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
