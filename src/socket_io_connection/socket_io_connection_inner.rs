use std::collections::HashMap;

use super::*;

pub struct SocketIoConnectionInner {
    pub active_ack_awaiters: HashMap<String, AckAwaiters>,
    ack_id: i64,
    pub sid: Option<String>,
}

impl SocketIoConnectionInner {
    pub fn new() -> Self {
        SocketIoConnectionInner {
            active_ack_awaiters: HashMap::new(),
            ack_id: 0,
            sid: None,
        }
    }

    pub fn get_next_ack_id(&mut self) -> i64 {
        self.ack_id += 1;
        self.ack_id
    }
}
