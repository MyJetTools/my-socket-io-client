use std::sync::Arc;

use my_web_socket_client::*;
use rust_extensions::{Logger, StrOrString};

use crate::{client_inner::ClientInner, SocketIoCallbacks};

pub struct MySocketIoClient {
    ws_client: WebSocketClient,
    inner: Arc<ClientInner>,
}

impl MySocketIoClient {
    pub fn new(
        name: impl Into<StrOrString<'static>>,
        settings: Arc<dyn WsClientSettings + Send + Sync + 'static>,
        callbacks: Arc<dyn SocketIoCallbacks + Send + Sync + 'static>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) -> Self {
        let ws_client = WebSocketClient::new(name, settings, logger);

        MySocketIoClient {
            ws_client,
            inner: Arc::new(ClientInner::new(callbacks)),
        }
    }

    pub fn set_debug_payloads(self, debug_payloads: bool) -> Self {
        self.inner
            .debug_payloads
            .store(debug_payloads, std::sync::atomic::Ordering::Relaxed);
        self
    }

    pub fn start(&self) {
        self.ws_client.start(None, self.inner.clone());
    }
}
