use std::{collections::HashMap, sync::Arc};

use rust_extensions::TaskCompletion;

use crate::SocketIoNamespaceCallbacks;

use super::*;

pub struct SocketIoConnectionInner {
    pub subscribers:
        HashMap<&'static str, Arc<dyn SocketIoNamespaceCallbacks + Send + Sync + 'static>>,

    pub active_subscribe_awaiters: HashMap<String, TaskCompletion<(), String>>,

    pub active_ack_awaiters: HashMap<String, AckAwaiters>,
    ack_id: u64,
}

impl SocketIoConnectionInner {
    pub fn new() -> Self {
        SocketIoConnectionInner {
            subscribers: HashMap::new(),
            active_subscribe_awaiters: HashMap::new(),
            active_ack_awaiters: HashMap::new(),
            ack_id: 0,
        }
    }

    pub fn get_next_ack_id(&mut self) -> u64 {
        self.ack_id += 1;
        self.ack_id
    }
}
