use std::collections::HashMap;

use rust_extensions::TaskCompletion;

pub struct AckAwaiters {
    awaiters: HashMap<i64, TaskCompletion<String, String>>,
}

impl AckAwaiters {
    pub fn new() -> Self {
        AckAwaiters {
            awaiters: HashMap::new(),
        }
    }

    pub fn add_awaiter(&mut self, ack_id: i64, task_completion: TaskCompletion<String, String>) {
        self.awaiters.insert(ack_id, task_completion);
    }

    pub fn remove_awaiter(&mut self, ack_id: i64) -> Option<TaskCompletion<String, String>> {
        self.awaiters.remove(&ack_id)
    }
}
