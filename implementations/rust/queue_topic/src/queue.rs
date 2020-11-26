use std::collections::HashMap;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

pub struct QueueMessage {
    pub body: Vec<u8>,
}

impl QueueMessage {
    pub fn new(body: Vec<u8>) -> QueueMessage {
        QueueMessage {
            body
        }
    }
}

pub trait Queue {
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

pub struct MemQueueWorker {
   queue_map: HashMap<String, Rc<RefCell<Box<dyn Queue>>>>
}

pub trait QueueWorker {
    fn get_queue(&mut self, queue_address: &str) -> Option<Rc<RefCell<Box<dyn Queue>>>>;

    fn remove_queue(&mut self, queue_name: &str);
}

impl MemQueueWorker {
    pub fn new() -> MemQueueWorker {
        MemQueueWorker {
            queue_map: HashMap::new()
        }
    }
}

impl QueueWorker for MemQueueWorker {

    fn get_queue(&mut self, queue_address: &str) -> Option<Rc<RefCell<Box<dyn Queue>>>> {
        if self.queue_map.contains_key(queue_address) {
            self.queue_map.get(queue_address).cloned()
        } else {
            let name_string = queue_address.to_string();
            let new_queue : Rc<RefCell<Box<dyn Queue>>> = Rc::new(RefCell::new(Box::new(MemQueue::new(name_string.clone()))));
            self.queue_map.insert(name_string.clone(), new_queue);
            self.queue_map.get(queue_address).cloned()
        }
    }

    fn remove_queue(&mut self, queue_name: &str) {
        self.queue_map.remove(queue_name);
    }
}

#[test]
fn queue_tdd() {
    let queue_worker = RefCell::new(MemQueueWorker::new());
    let queue_address = "test";
    {
        let mut qw = queue_worker.borrow_mut();
        let mut queue: Rc<RefCell<Box<dyn Queue>>> = qw.get_queue(queue_address).unwrap();

        let mut rm = queue.borrow_mut();
        rm.enqueue(QueueMessage::new("A".as_bytes().to_vec()));
    //    queue.enqueue(QueueMessage::new("B".as_bytes().to_vec()));
//        queue.enqueue(QueueMessage::new("C".as_bytes().to_vec()));

      //  let out_message = queue.dequeue().unwrap();
     //   println!("{:?}", out_message.body);
    }

  /*  let mut qw = queue_worker.borrow_mut();
    qw.borrow_mut().remove_queue(queue_address);*/
}
