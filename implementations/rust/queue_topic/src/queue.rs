use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::borrow::{Borrow, BorrowMut};

pub struct QueueMessage {
    pub body: Vec<u8>,
}

impl QueueMessage {
    fn new(body: Vec<u8>) -> QueueMessage {
        QueueMessage {
            body
        }
    }
}

trait Queue {
    fn address(&self) -> String;
    fn enqueue(&mut self, message: QueueMessage);
    fn dequeue(&mut self) -> Option<QueueMessage>;
}

struct MemQueue {
    address: String,
    messages: Vec<QueueMessage>
}

impl MemQueue {
    fn new<S>(address: S) -> MemQueue
    where
        S: ToString,
    {
        MemQueue {
            address: address.to_string(),
            messages: Vec::new()
        }
    }
}

impl Queue for MemQueue {
    fn address(&self) -> String {
        self.address.clone()
    }

    fn enqueue(&mut self, message: QueueMessage) {
        self.messages.push(message);
    }

    fn dequeue(&mut self) -> Option<QueueMessage> {
        match self.messages.len() == 0 {
            false => Some(self.messages.remove(0)),
            true => None
        }
    }
}

struct MemQueueWorker {
   queue_map: HashMap<String, RefCell<Box<dyn Queue>>>
}

impl MemQueueWorker {
    fn new() -> MemQueueWorker {
        MemQueueWorker {
            queue_map: HashMap::new()
        }
    }

    fn get_queue(&mut self, queue_name: &str) -> Option<&RefCell<Box<dyn Queue>>> {
        if self.queue_map.contains_key(queue_name) {
            self.queue_map.get(queue_name)
        } else {
            let name_string = queue_name.to_string();
            let new_queue = RefCell::new(Box::new(MemQueue::new(name_string.clone())));
            self.queue_map.insert(name_string.clone(), new_queue);
            self.queue_map.get(queue_name)
        }
    }

    fn remove_queue(&mut self, queue_name: &str) {
        self.queue_map.remove(queue_name);
    }
}

#[test]
fn queue_tdd() {
    let queue_worker = RefCell::new(MemQueueWorker::new());
    let queue_name = "test";
    {
        let mut qw = queue_worker.borrow_mut();
        let mut queue = qw.get_queue(queue_name).unwrap().borrow_mut();

        queue.enqueue(QueueMessage::new("A".as_bytes().to_vec()));
        queue.enqueue(QueueMessage::new("B".as_bytes().to_vec()));
        queue.enqueue(QueueMessage::new("C".as_bytes().to_vec()));

        let out_message = queue.dequeue().unwrap();
        println!("{:?}", out_message.body);
    }

    let mut qw = queue_worker.borrow_mut();
    qw.borrow_mut().remove_queue(queue_name);
}
