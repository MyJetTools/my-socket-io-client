### How to use it

```rust

pub struct AppSettings{
    pub my_socket_io_url: String,
}

#[async_trait::async_trait]
impl WsClientSettings for AppSettings {
    async fn get_url(&self, client_name: &str) -> String {

        if client_name == "my-client-name"{
            return self.my_socket_io_url.to_string();
        }

        panic!("Unknown socket-io client: '{}'", client_name);
    }
}

```


## How to setup subscriber

This example subscribes to the stream event and deserializes the payload based on the `type` field.

```rust
use my_socket_io_client::*;
use serde::*;

pub struct StreamsSocketIo;

#[derive(Debug)]
pub enum SocketIoStreamModel {
    AccountType(AccountTypeSocketIoModel),
    Property(PropertySocketIoModel),
}

impl SocketIoSubscribeEventModel for SocketIoStreamModel {
    const NAME_SPACE: &'static str = "/brand-socket";

    const EVENT_NAME: &'static str = "stream";

    fn deserialize(payload: &str) -> Self {
        let type_model: StreamTypeModel = serde_json::from_str(payload).unwrap();

        match type_model.r#type.as_str() {
            "AccountStatus" => Self::AccountType(serde_json::from_str(payload).unwrap()),
            "Property" => Self::Property(serde_json::from_str(payload).unwrap()),
            _ => {
                panic!("Unknown stream type: {}", type_model.r#type);
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamTypeModel {
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountTypeSocketIoModel {
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub currency: String,
    pub balance: Option<String>,
    #[serde(rename = "marginAvailable")]
    pub margin_available: Option<String>,
    pub credit: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertySocketIoModel {
    pub name: String,
}

#[async_trait::async_trait]
impl SocketIoEventSubscriberCallback<SocketIoStreamModel, ()> for StreamsSocketIo {
    async fn on_event(&self, event_payload: SocketIoStreamModel) -> () {
        println!("Received event: {:?}", event_payload);
        ()
    }
}



```



### Setup  socket-io client

```rust
use std::sync::Arc;

use my_socket_io_client::*;
use streams_socket_io::StreamsSocketIo;

mod streams_socket_io;

#[tokio::main]
async fn main() {
    let settings = Arc::new(AppSettings);

    my_web_socket_client::my_tls::install_default_crypto_providers();

    let callbacks = Arc::new(AppSocketIoCallbacks);
    let socket_io_client = MySocketIoClient::new(
        "my-client-name",
        settings,
        callbacks,
        my_logger::LOGGER.clone(),
    )
    .set_debug_payloads(true);

    socket_io_client
        .register_subscriber(Arc::new(StreamsSocketIo))
        .await;

    socket_io_client.start();
    println!("Starting");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}


pub struct AppSocketIoCallbacks;

#[async_trait::async_trait]
impl SocketIoCallbacks for AppSocketIoCallbacks {
    async fn before_connect(&self) -> SocketIoBeforeConnectResult {
        SocketIoBeforeConnectResult {
            append_headers: vec![("brand-api-key".into(), "key".into())].into(),
            //append_headers: None,
            append_query_params: vec![("type".into(), "LIVE".into())].into(),
        }
    }
    async fn on_connect(&self, _socket: Arc<SocketIoConnection>) {
        println!("Connected to Socket-Io");
    }
    async fn on_disconnect(&self, _socket: Arc<SocketIoConnection>) {
        println!("Disconnected from Socket-Io");
    }
}


```