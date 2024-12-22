use std::sync::Arc;

use my_web_socket_client::*;
use rust_extensions::{Logger, StrOrString};

use crate::{client_inner::*, *};

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
        let name = name.into();
        let name = Arc::new(name);
        let ws_client = WebSocketClient::new(name.clone(), settings, logger.clone());

        MySocketIoClient {
            ws_client,
            inner: Arc::new(ClientInner::new(name, callbacks, logger)),
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

    pub async fn register_subscriber<
        TModel: SocketIoSubscribeEventModel + Send + Sync + 'static,
        TOutModel: SocketIoSubscribeOutModel + Send + Sync + 'static + serde::Serialize,
    >(
        &self,
        callbacks: Arc<
            dyn SocketIoEventSubscriberCallback<TModel, TOutModel> + Send + Sync + 'static,
        >,
    ) {
        let subscriber = SocketIoEventSubscriber { callbacks };
        let subscriber = Arc::new(subscriber);
        self.inner
            .event_subscribers
            .register(TModel::NAME_SPACE, TModel::EVENT_NAME, subscriber)
            .await;
    }
}
