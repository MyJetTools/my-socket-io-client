use socket_io_utils::{SocketIoContract, SocketIoHandshakeOpenModel, SocketIoMessage};
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::Mutex;

use my_web_socket_client::{
    hyper_tungstenite::tungstenite::Message, url_utils::UrlBuilder, StartWsConnectionDataToApply,
    WsCallback, WsConnection,
};

use crate::{SocketIoCallbacks, SocketIoConnection};

#[derive(Default)]
pub struct SocketIoContext {
    pub handshake_response: Option<SocketIoHandshakeOpenModel>,
    pub current_connection: Option<Arc<SocketIoConnection>>,
}

pub struct ClientInner {
    callbacks: Arc<dyn SocketIoCallbacks + Send + Sync + 'static>,
    context: Mutex<SocketIoContext>,
    pub debug_payloads: AtomicBool,
}

impl ClientInner {
    pub fn new(callbacks: Arc<dyn SocketIoCallbacks + Send + Sync + 'static>) -> Self {
        ClientInner {
            callbacks,
            context: Mutex::new(SocketIoContext::default()),
            debug_payloads: AtomicBool::new(false),
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
        match message {
            SocketIoMessage::Connect { namespace, sid: _ } => {
                let connection: Arc<SocketIoConnection> = self.get_current_connection().await;
                connection.subscribe_confirmed(namespace.as_str()).await;
            }
            SocketIoMessage::Disconnect { namespace: _ } => {}
            SocketIoMessage::Event {
                namespace,
                data,
                ack,
            } => {
                let connection: Arc<SocketIoConnection> = self.get_current_connection().await;
                let response = self
                    .callbacks
                    .on_socket_io_event(&connection, namespace.as_str(), data)
                    .await;

                if let Some(ack) = ack {
                    let ack = SocketIoMessage::Ack {
                        namespace,
                        data: response.unwrap_or_default(),
                        ack,
                    }
                    .into();

                    connection.send_message(&ack).await;
                }
            }
            SocketIoMessage::Ack {
                namespace,
                data,
                ack,
            } => {
                let connection: Arc<SocketIoConnection> = self.get_current_connection().await;
                connection
                    .handle_ack_event(namespace.as_str(), ack, data)
                    .await;
            }
            SocketIoMessage::ConnectError { namespace, message } => {
                let connection: Arc<SocketIoConnection> = self.get_current_connection().await;
                connection
                    .subscribe_failed(namespace.as_str(), message.to_string())
                    .await;
            }
        }
    }

    async fn handle_socket_io_contract(&self, contract: SocketIoContract) {
        match contract {
            SocketIoContract::Open(model) => {
                let mut context = self.context.lock().await;
                context.handshake_response = Some(model);
            }
            SocketIoContract::Close => {
                let connection = self.get_current_connection().await;
                connection.disconnect().await;
            }
            SocketIoContract::Ping { with_probe } => {
                let connection = self.get_current_connection().await;
                let pong = SocketIoContract::Pong { with_probe };
                connection.send_message(&pong).await;
            }
            SocketIoContract::Pong { with_probe: _ } => {}
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
        let before_connect_result = self.callbacks.before_connect().await;

        let mut url = UrlBuilder::new(url.as_str());
        url.append_query_param("EIO", Some("4"));
        url.append_query_param("transport", Some("websocket"));

        let result = StartWsConnectionDataToApply {
            headers: before_connect_result.headers,
            url: Some(url),
        };

        Ok(result)
    }
    async fn on_connected(&self, ws_connection: Arc<WsConnection>) {
        let connection = SocketIoConnection::new(ws_connection, self.get_debug_payloads());
        let connection = Arc::new(connection);

        self.set_current_connection(connection.clone()).await;

        let callbacks = self.callbacks.clone();
        let _ = tokio::spawn(async move {
            callbacks.on_connect(connection).await;
        })
        .await;
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
        match data {
            Message::Text(test) => {
                if debug_payloads {
                    print!("Socket IO Text ws message received: {}", test);
                }

                let contract = SocketIoContract::deserialize(test.as_str());
                self.handle_socket_io_contract(contract).await;
            }
            Message::Binary(_) => {
                panic!("Binary Payloads are not supported yet");
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
