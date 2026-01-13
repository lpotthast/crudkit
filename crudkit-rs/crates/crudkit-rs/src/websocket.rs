use crudkit_websocket::CkWsMessage;
use uuid::Uuid;

/// We assume that users establish a websocket connection.
///
/// This trait allows crudkit to communicate with specific or all users through websocket messages.
///
/// It is used to send validation status updates.
pub trait CrudWebsocketService {
    type Error;

    /// Returns the ID of the user executing the current request in the websocket system.
    fn current_user_id(&self) -> Uuid;

    /// Send a message to one specific user.
    fn send_json(&self, to: Uuid, json: CkWsMessage) -> Result<(), Self::Error>;

    /// Send a message to all users.
    fn broadcast_json(&self, json: CkWsMessage) -> Result<(), Self::Error>;
}
