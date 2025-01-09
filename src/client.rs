use std::sync::Arc;

use my_web_socket_client::*;
use my_web_socket_client::hyper_tungstenite::tungstenite::Message;
use rust_extensions::{Logger, StrOrString};

use crate::{client_inner::*, *};

pub struct MySocketIoClient {
    ws_client: WebSocketClient,
    inner: Arc<ClientInner>,
}

impl MySocketIoClient {
    pub fn new(
        name: impl Into<StrOrString<'static>>,
        settings: Arc<dyn SocketIoClientSettings + Send + Sync + 'static>,
        callbacks: Arc<dyn SocketIoCallbacks + Send + Sync + 'static>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) -> Self {
        let settings = WebSocketIoSettings {
            socket_io_settings: settings,
        };

        let settings = Arc::new(settings);

        let name = name.into();
        let name = Arc::new(name);
        let ws_client = WebSocketClient::new(name.clone(), settings.clone(), logger.clone());

        MySocketIoClient {
            ws_client,
            inner: Arc::new(ClientInner::new(name, callbacks, settings, logger)),
        }
    }

    pub fn set_debug_payloads(self, debug_payloads: bool) -> Self {
        self.inner
            .debug_payloads
            .store(debug_payloads, std::sync::atomic::Ordering::Relaxed);
        self
    }

    pub fn start(&self) {
        let ping_message = Message::Ping(bytes::Bytes::new());
        self.ws_client.start(Some(ping_message), self.inner.clone());
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

    pub fn stop(&self) {
        self.ws_client.stop()
    }
}

pub(crate) struct WebSocketIoSettings {
    pub socket_io_settings: Arc<dyn SocketIoClientSettings + Send + Sync + 'static>,
}

#[async_trait::async_trait]
impl WsClientSettings for WebSocketIoSettings {
    async fn get_url(&self, client_name: &str) -> String {
        let mut result = self.socket_io_settings.get_server_url(client_name).await;

        if !result.ends_with('/') {
            result.push('/');
        }

        let handshake_path = self
            .socket_io_settings
            .get_handshake_path(client_name)
            .await;

        if handshake_path.starts_with('/') {
            result.push_str(&handshake_path[1..]);
        } else {
            result.push_str(&handshake_path);
        }

        if !result.ends_with('/') {
            result.push('/');
        }

        result
    }
}
