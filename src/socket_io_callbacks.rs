use std::sync::Arc;

use socket_io_utils::SocketIoEventParameter;

use crate::SocketIoConnection;

#[derive(Default)]
pub struct SocketIoBeforeConnectResult {
    pub headers: Option<Vec<(String, String)>>,
}

#[async_trait::async_trait]
pub trait SocketIoCallbacks {
    async fn before_connect(&self) -> SocketIoBeforeConnectResult;
    async fn on_connect(&self, socket: Arc<SocketIoConnection>);
    async fn on_disconnect(&self, socket: Arc<SocketIoConnection>);

    async fn on_socket_io_event(
        &self,
        socket: &Arc<SocketIoConnection>,
        namespace: &str,
        data: Vec<SocketIoEventParameter>,
    ) -> Option<Vec<SocketIoEventParameter>>;
}
