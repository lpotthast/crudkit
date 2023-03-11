// TODO: Get rid of this? Use aggregate specific data all the way. See context.rs and the repository field as an example.
// pub struct CrudController {
//     ws: Arc<dyn WebsocketController<Json = crudkit_websocket::CkWsMessage>>,
// }

// impl CrudController {
//     pub fn new(ws: Arc<dyn WebsocketController<Json = crudkit_websocket::CkWsMessage>>) -> Self {
//         Self { ws }
//     }

//     pub fn get_websocket_controller(
//         &self,
//     ) -> &dyn WebsocketController<Json = crudkit_websocket::CkWsMessage> {
//         self.ws.as_ref()
//     }
// }
