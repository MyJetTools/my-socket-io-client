use std::sync::Arc;

use my_web_socket_client::{hyper_tungstenite::tungstenite::Message, WsConnection};
use rust_extensions::{StrOrString, TaskCompletion};
use socket_io_utils::{SocketIoContract, SocketIoEventParameter, SocketIoMessage};
use tokio::sync::Mutex;

use super::*;
use crate::SocketIoNamespaceCallbacks;
pub struct SocketIoConnection {
    ws_connection: Arc<WsConnection>,
    inner: Mutex<SocketIoConnectionInner>,
    debug_payload: bool,
}

impl SocketIoConnection {
    pub fn new(ws_connection: Arc<WsConnection>, debug_payload: bool) -> Self {
        SocketIoConnection {
            ws_connection,
            inner: Mutex::new(SocketIoConnectionInner::new()),
            debug_payload,
        }
    }

    pub async fn subscribe_to_namespace(
        &self,
        callback: Arc<dyn SocketIoNamespaceCallbacks + Send + Sync + 'static>,
    ) {
        let namespace = callback.get_namespace();
        {
            let mut inner = self.inner.lock().await;
            inner.subscribers.insert(namespace, callback);
        }

        let contract: SocketIoContract = SocketIoMessage::Connect {
            namespace: namespace.to_string().into(),
            sid: None,
        }
        .into();

        self.send_message(&contract).await;
    }

    pub async fn subscribe_confirmed(&self, namespace: &str) {
        let mut inner = self.inner.lock().await;
        let mut task_completion = inner.active_subscribe_awaiters.remove(namespace).unwrap();
        task_completion.set_ok(());
    }

    pub async fn subscribe_failed(&self, namespace: &str, message: String) {
        let mut inner = self.inner.lock().await;
        let mut task_completion = inner.active_subscribe_awaiters.remove(namespace).unwrap();
        task_completion.set_error(message);
    }

    pub async fn send_message(&self, contract: &SocketIoContract) {
        let payload = contract.serialize();

        if payload.binary_frames.len() == 0 {
            if self.debug_payload {
                println!("Sending socket_io text payload: '{:?}'", payload.text_frame);
            }

            self.ws_connection
                .send_message(Message::Text(payload.text_frame.into()))
                .await;
        } else {
            if self.debug_payload {
                println!("Sending socket_io text payload: '{:?}'", payload.text_frame);
            }
            let mut payloads = vec![Message::Text(payload.text_frame.into())];

            for binary in payload.binary_frames {
                println!("Sending socket_io binary payload: Len:{}", binary.len());
                payloads.push(Message::Binary(binary.into()));
            }

            self.ws_connection.send_messages(payloads.into_iter()).await;
        }
    }

    pub async fn send_event_with_ack(
        &self,
        namespace: impl Into<StrOrString<'static>>,
        parameters: Vec<SocketIoEventParameter>,
    ) -> Result<Vec<SocketIoEventParameter>, String> {
        let (awaiter, message) = {
            let mut inner = self.inner.lock().await;

            let ack_id = inner.get_next_ack_id();
            let namespace = namespace.into();

            let mut task_completion = TaskCompletion::new();

            let awaiter = task_completion.get_awaiter();
            match inner.active_ack_awaiters.get_mut(namespace.as_str()) {
                Some(awaiters) => {
                    awaiters.add_awaiter(ack_id, task_completion);
                }
                None => {
                    let mut awaiters = AckAwaiters::new();
                    awaiters.add_awaiter(ack_id, task_completion);
                    inner
                        .active_ack_awaiters
                        .insert(namespace.as_str().to_string(), awaiters);
                }
            }

            let message = SocketIoMessage::Event {
                namespace,
                data: parameters,
                ack: ack_id.into(),
            };

            (awaiter, message)
        };

        self.send_message(&message.into()).await;

        awaiter.get_result().await
    }

    pub async fn send_event_and_forget(
        &self,
        namespace: impl Into<StrOrString<'static>>,
        parameters: Vec<SocketIoEventParameter>,
    ) {
        let message = SocketIoMessage::Event {
            namespace: namespace.into(),
            data: parameters,
            ack: None,
        };

        self.send_message(&message.into()).await;
    }

    pub(crate) async fn handle_ack_event(
        &self,
        namespace: &str,
        ack_id: u64,
        parameters: Vec<SocketIoEventParameter>,
    ) {
        let mut inner = self.inner.lock().await;

        if let Some(awaiters) = inner.active_ack_awaiters.get_mut(namespace) {
            if let Some(mut awaiter) = awaiters.remove_awaiter(ack_id) {
                awaiter.set_ok(parameters);
            } else {
                panic!("Trying to ack an event that has no awaiter");
            }
        } else {
            panic!("Trying to ack an event that has no namespace to find awaiter");
        }
    }

    pub async fn disconnect(&self) {
        self.ws_connection.disconnect().await;
    }
}
