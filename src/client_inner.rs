use rust_extensions::{Logger, StrOrString};
use socket_io_utils::{SocketIoContract, SocketIoHandshakeOpenModel, SocketIoMessage};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
};
use tokio::sync::Mutex;

use my_web_socket_client::{
    hyper_tungstenite::tungstenite::Message, url_utils::UrlBuilder, StartWsConnectionDataToApply,
    WsCallback, WsConnection,
};

use crate::{EventSubscribers, SocketIoCallbacks, SocketIoConnection};

#[derive(Default)]
pub struct SocketIoContext {
    pub handshake_response: Option<SocketIoHandshakeOpenModel>,
    pub current_connection: Option<Arc<SocketIoConnection>>,
}

pub struct ClientInner {
    client_name: Arc<StrOrString<'static>>,
    callbacks: Arc<dyn SocketIoCallbacks + Send + Sync + 'static>,
    context: Mutex<SocketIoContext>,
    pub debug_payloads: AtomicBool,
    pub event_subscribers: EventSubscribers,
    logger: Arc<dyn Logger + Send + Sync + 'static>,
}

impl ClientInner {
    pub fn new(
        client_name: Arc<StrOrString<'static>>,
        callbacks: Arc<dyn SocketIoCallbacks + Send + Sync + 'static>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) -> Self {
        ClientInner {
            client_name,
            callbacks,
            context: Mutex::new(SocketIoContext::default()),
            debug_payloads: AtomicBool::new(false),
            event_subscribers: EventSubscribers::new(),
            logger,
        }
    }

    pub fn get_debug_payloads(&self) -> bool {
        self.debug_payloads
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    async fn set_current_connection(&self, connection: Arc<SocketIoConnection>) {
        let mut context = self.context.lock().await;
        context.current_connection = Some(connection);
    }

    async fn get_current_connection(&self) -> Arc<SocketIoConnection> {
        let context = self.context.lock().await;
        context.current_connection.clone().unwrap()
    }

    async fn remote_current_connection(&self) -> Arc<SocketIoConnection> {
        let mut context = self.context.lock().await;
        context.handshake_response = None;
        context.current_connection.take().unwrap()
    }

    async fn handle_socket_io_message(&self, message: SocketIoMessage) {
        const PROCESS: &'static str = "handle_socket_io_message";
        match message {
            SocketIoMessage::Connect { namespace, sid: _ } => {
                let mut ctx = HashMap::new();
                ctx.insert("namespace".to_string(), namespace.to_string());
                ctx.insert("name".to_string(), self.client_name.as_str().to_string());

                self.logger.write_info(
                    PROCESS.to_string(),
                    "Connected to namespace".to_string(),
                    Some(ctx),
                );
            }
            SocketIoMessage::Disconnect { namespace: _ } => {}
            SocketIoMessage::Event {
                namespace,
                event_name,
                data,
                ack,
            } => {
                let connection: Arc<SocketIoConnection> = self.get_current_connection().await;

                if let Some(subscriber) = self
                    .event_subscribers
                    .get(namespace.as_str(), event_name.as_str())
                    .await
                {
                    let result = subscriber.on_event(data.as_str()).await;

                    if let Some(ack) = ack {
                        let ack = SocketIoMessage::Ack {
                            namespace,
                            event_name,
                            data: result.into(),
                            ack,
                        }
                        .into();

                        connection.send_message(&ack).await;
                    }
                }
            }
            SocketIoMessage::Ack {
                namespace,
                event_name: _,
                data,
                ack,
            } => {
                let connection: Arc<SocketIoConnection> = self.get_current_connection().await;
                connection
                    .handle_ack_event(namespace.as_str(), ack, data.to_string())
                    .await;
            }
            SocketIoMessage::ConnectError { namespace, message } => {
                let mut ctx = HashMap::new();
                ctx.insert("namespace".to_string(), namespace.to_string());
                ctx.insert("name".to_string(), self.client_name.as_str().to_string());

                self.logger.write_fatal_error(
                    PROCESS.to_string(),
                    format!("Connection to namespace failed. Msg: {}", message.as_str()),
                    Some(ctx),
                );
            }
        }
    }

    async fn handle_socket_io_contract(&self, contract: SocketIoContract) {
        match contract {
            SocketIoContract::Open(model) => {
                let connection = self.get_current_connection().await;

                connection.set_sid(model.sid.clone()).await;

                {
                    let mut context = self.context.lock().await;
                    context.handshake_response = Some(model);
                }

                println!("Handled Handshake");

                let namespaces = self.event_subscribers.get_namespaces().await;

                connection.subscribe_to_namespaces(namespaces).await;

                let callbacks = self.callbacks.clone();

                tokio::spawn(async move {
                    callbacks.on_connect(connection).await;
                }); //Never await it
            }
            SocketIoContract::Close => {
                let connection = self.get_current_connection().await;
                connection.disconnect().await;
            }
            SocketIoContract::Ping { with_probe } => {
                if !with_probe {
                    let connection = self.get_current_connection().await;
                    let pong = SocketIoContract::Pong { with_probe: false };
                    connection.send_message(&pong).await;
                }
            }
            SocketIoContract::Pong { with_probe } => {
                println!("Pong received with_probe: {}", with_probe);

                if with_probe {
                    let connection = self.get_current_connection().await;
                    connection.send_message(&SocketIoContract::Upgrade).await;
                }
            }
            SocketIoContract::Message(socket_io_message) => {
                self.handle_socket_io_message(socket_io_message).await;
            }
            SocketIoContract::Upgrade => {}
            SocketIoContract::Noop => {}
        }
    }
}
#[async_trait::async_trait]
impl WsCallback for ClientInner {
    async fn before_start_ws_connect(
        &self,
        url: String,
    ) -> Result<StartWsConnectionDataToApply, String> {
        let mut before_connect_result = self.callbacks.before_connect().await;

        let mut url = UrlBuilder::new(url.as_str());
        url.append_query_param("EIO", Some("4"));
        url.append_query_param("transport", Some("websocket"));

        if let Some(append_query_params) = before_connect_result.append_query_params.take() {
            for (key, value) in append_query_params {
                url.append_query_param(key.as_str(), Some(value.as_str()));
            }
        }

        let result = StartWsConnectionDataToApply {
            headers: before_connect_result.append_headers,
            url: Some(url),
        };

        Ok(result)
    }
    async fn on_connected(&self, ws_connection: Arc<WsConnection>) {
        let connection = SocketIoConnection::new(ws_connection, self.get_debug_payloads());
        let connection = Arc::new(connection);
        self.set_current_connection(connection.clone()).await;
    }
    async fn on_disconnected(&self, _ws_connection: Arc<WsConnection>) {
        let connection = self.remote_current_connection().await;

        let callbacks = self.callbacks.clone();
        let _ = tokio::spawn(async move {
            callbacks.on_disconnect(connection).await;
        })
        .await;
    }
    async fn on_data(&self, _ws_connection: Arc<WsConnection>, data: Message) {
        let debug_payloads = self.get_debug_payloads();

        println!("Data received {:?}", data);
        match data {
            Message::Text(text) => {
                if debug_payloads {
                    println!("Socket IO Text ws message received: {}", text);
                }

                let contract = SocketIoContract::deserialize(text.as_str());
                self.handle_socket_io_contract(contract).await;
            }
            Message::Binary(_) => {
                println!("Binary Payloads are not supported yet");
            }
            Message::Ping(payload) => {
                if debug_payloads {
                    println!("Socket IO Ping ws message received: len={}", payload.len());
                }
            }
            Message::Pong(payload) => {
                if debug_payloads {
                    println!("Socket IO Pong ws message received: len={}", payload.len());
                }
            }
            Message::Close(_) => {
                if debug_payloads {
                    println!("Socket IO Close ws message received");
                }
            }
            Message::Frame(_) => {
                if debug_payloads {
                    println!(
                        "Socket IO Frame ws message received. Debug message. Ask to remove it"
                    );
                }
            }
        }
    }
}
