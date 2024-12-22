use std::sync::Arc;

use rust_extensions::StrOrString;

use crate::SocketIoConnection;

#[derive(Default)]
pub struct SocketIoBeforeConnectResult {
    pub append_headers: Option<Vec<(StrOrString<'static>, StrOrString<'static>)>>,
    pub append_query_params: Option<Vec<(StrOrString<'static>, StrOrString<'static>)>>,
}

#[async_trait::async_trait]
pub trait SocketIoCallbacks {
    async fn before_connect(&self) -> SocketIoBeforeConnectResult;
    async fn on_connect(&self, socket: Arc<SocketIoConnection>);
    async fn on_disconnect(&self, socket: Arc<SocketIoConnection>);
}
