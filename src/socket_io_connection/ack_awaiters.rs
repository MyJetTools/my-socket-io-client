use std::collections::HashMap;

use rust_extensions::TaskCompletion;
use socket_io_utils::SocketIoEventParameter;

pub struct AckAwaiters {
    awaiters: HashMap<u64, TaskCompletion<Vec<SocketIoEventParameter>, String>>,
}

impl AckAwaiters {
    pub fn new() -> Self {
        AckAwaiters {
            awaiters: HashMap::new(),
        }
    }

    pub fn add_awaiter(
        &mut self,
        ack_id: u64,
        task_completion: TaskCompletion<Vec<SocketIoEventParameter>, String>,
    ) {
        self.awaiters.insert(ack_id, task_completion);
    }

    pub fn remove_awaiter(
        &mut self,
        ack_id: u64,
    ) -> Option<TaskCompletion<Vec<SocketIoEventParameter>, String>> {
        self.awaiters.remove(&ack_id)
    }
}
