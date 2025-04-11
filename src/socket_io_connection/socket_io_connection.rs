use std::sync::Arc;

use my_web_socket_client::{hyper_tungstenite::tungstenite::Message, WsConnection};
use rust_extensions::TaskCompletion;
use socket_io_utils::{SocketIoContract, SocketIoMessage};
use tokio::sync::Mutex;

use super::*;
use crate::*;
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
    pub async fn set_sid(&self, sid: String) {
        let mut inner = self.inner.lock().await;
        inner.sid = Some(sid);
    }

    pub async fn get_sid(&self) -> Option<String> {
        let inner = self.inner.lock().await;
        inner.sid.clone()
    }

    pub(crate) async fn subscribe_to_namespaces(&self, namespaces: Vec<&'static str>) {
        for namespace in namespaces {
            let contract: SocketIoContract = SocketIoMessage::Connect {
                namespace: namespace.into(),
                sid: None,
            }
            .into();

            self.send_message(&contract).await;
        }
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
                if self.debug_payload {
                    println!("Sending socket_io binary payload: Len:{}", binary.len());
                }
                payloads.push(Message::Binary(binary.into()));
            }

            self.ws_connection.send_messages(payloads.into_iter()).await;
        }
    }

    pub async fn send_event_with_ack<
        TInModel: SocketIoRpcInModel,
        TOutModel: SocketIoRpcOutModel,
    >(
        &self,
        data: &TInModel,
    ) -> Result<TOutModel, String> {
        let (awaiter, message) = {
            let mut inner = self.inner.lock().await;

            let ack_id = inner.get_next_ack_id();

            let mut task_completion = TaskCompletion::new();

            let awaiter = task_completion.get_awaiter();

            match inner.active_ack_awaiters.get_mut(TInModel::NAME_SPACE) {
                Some(awaiters) => {
                    awaiters.add_awaiter(ack_id, task_completion);
                }
                None => {
                    let mut awaiters = AckAwaiters::new();
                    awaiters.add_awaiter(ack_id, task_completion);
                    inner
                        .active_ack_awaiters
                        .insert(TInModel::NAME_SPACE.to_string(), awaiters);
                }
            }

            let data = data.serialize();

            let message = SocketIoMessage::Event {
                namespace: TInModel::NAME_SPACE.into(),
                data: data.into(),
                event_name: TInModel::EVENT_NAME.into(),
                ack: ack_id.into(),
            };

            (awaiter, message)
        };

        self.send_message(&message.into()).await;

        let result = awaiter.get_result().await?;

        let result = TOutModel::deserialize(&result);
        Ok(result)
    }

    pub async fn send_event_and_forget<TInModel: SocketIoRpcInModel>(&self, model: &TInModel) {
        let data = model.serialize();

        let message = SocketIoMessage::Event {
            namespace: TInModel::NAME_SPACE.into(),
            event_name: TInModel::EVENT_NAME.into(),
            data: data.into(),
            ack: None,
        };

        self.send_message(&message.into()).await;
    }

    pub(crate) async fn handle_ack_event(&self, namespace: &str, ack_id: i64, data: String) {
        let mut inner = self.inner.lock().await;

        if let Some(awaiters) = inner.active_ack_awaiters.get_mut(namespace) {
            if let Some(mut awaiter) = awaiters.remove_awaiter(ack_id) {
                awaiter.set_ok(data);
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
