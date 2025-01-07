use std::sync::Arc;

use crate::SocketIoConnection;

#[async_trait::async_trait]
pub trait SocketIoCallbacks {
    async fn on_connect(&self, socket: Arc<SocketIoConnection>);
    async fn on_disconnect(&self, socket: Arc<SocketIoConnection>);
}
