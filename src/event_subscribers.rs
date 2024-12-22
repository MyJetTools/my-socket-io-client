use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::*;

pub struct EventSubscribers {
    items: Mutex<
        HashMap<
            &'static str,
            HashMap<
                &'static str,
                Arc<dyn SocketIoEventSubscriberNotification + Send + Sync + 'static>,
            >,
        >,
    >,
}

impl EventSubscribers {
    pub fn new() -> Self {
        EventSubscribers {
            items: Mutex::new(HashMap::new()),
        }
    }

    pub async fn register(
        &self,
        namespace: &'static str,
        event_name: &'static str,
        subscriber: Arc<dyn SocketIoEventSubscriberNotification + Send + Sync + 'static>,
    ) {
        let mut items = self.items.lock().await;

        if !items.contains_key(namespace) {
            items.insert(namespace, HashMap::new());
        }

        let subscribers = items.get_mut(namespace).unwrap();

        if subscribers.contains_key(event_name) {
            panic!(
                "Subscriber to namespace {} and event {} already exists",
                namespace, event_name
            );
        }

        subscribers.insert(event_name, subscriber);
    }

    pub async fn get(
        &self,
        namespace: &str,
        event_name: &str,
    ) -> Option<Arc<dyn SocketIoEventSubscriberNotification + Send + Sync + 'static>> {
        let items = self.items.lock().await;
        let subscribers = items.get(namespace)?;
        let subscriber = subscribers.get(event_name)?;

        Some(subscriber.clone())
    }

    pub async fn get_namespaces(&self) -> Vec<&'static str> {
        let items = self.items.lock().await;
        items.keys().map(|k| *k).collect()
    }
}
