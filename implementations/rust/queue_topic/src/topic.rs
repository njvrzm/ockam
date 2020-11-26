use std::collections::HashMap;
use std::rc::Rc;
use crate::queue::*;
use std::cell::RefCell;

pub trait Topic {
    fn topic_address(&self) -> String;
}

pub struct MemTopic {
    pub topic_address: String,
}

impl MemTopic {
    pub fn new(topic_name: String) -> TopicHandle {
        Rc::new(Box::new(MemTopic { topic_address: topic_name }))
    }
}

impl Topic for MemTopic {
    fn topic_address(&self) -> String {
        self.topic_address.clone()
    }
}

pub type TopicHandle = Rc<Box<dyn Topic>>;
pub type QueueHandle = Rc<Box<dyn Queue>>;

pub struct TopicSubscription {
    pub topic: TopicHandle,
    pub queue: QueueHandle,
    pub subscriber_address: String,
}

pub trait Subscription {
    fn topic(&self) -> TopicHandle;

    fn queue(&self) -> QueueHandle;

    fn subscriber_address(&self) -> String;
}

impl Subscription for TopicSubscription {
    fn topic(&self) -> TopicHandle {
        self.topic.clone()
    }

    fn queue(&self) -> QueueHandle {
        self.queue.clone()
    }

    fn subscriber_address(&self) -> String {
        self.subscriber_address.clone()
    }
}

pub trait TopicWorker {
    fn publish(&mut self, topic: &str, message: QueueMessage);

    fn subscribe(&mut self, topic: &str) -> Option<String>;

    fn poll(&mut self, subscriber: &str, message_handler: &dyn Fn(QueueMessage));

    fn unsubscribe(&mut self, subscriber: &str);
}

struct MemTopicWorker {
    queue_worker: RefCell<Box<dyn QueueWorker>>,
    subscriptions: HashMap<String, Rc<RefCell<Box<dyn Queue>>>>,
    subscription_id_counter: usize,
}

impl MemTopicWorker {
    fn new(queue_worker: RefCell<Box<dyn QueueWorker>>) -> MemTopicWorker {
        MemTopicWorker {
            subscriptions: HashMap::new(),
            subscription_id_counter: 0,
            queue_worker
        }
    }
}

impl TopicWorker for MemTopicWorker {
    fn publish(&mut self, topic: &str, message: QueueMessage) {
        unimplemented!()
    }

    fn subscribe(&mut self, topic_address: &str) -> Option<String> {
        let subscriber_address = format!("{}_{}", self.subscription_id_counter, topic_address);
        println!("{}", subscriber_address);

        match self.queue_worker.borrow_mut().get_queue(subscriber_address.as_str()) {
            Some(queue) => {
                self.subscriptions.insert(subscriber_address.clone(), queue);
                self.subscription_id_counter += 1;
                Some(subscriber_address.clone())
            },
            None => None
        }
    }

    fn poll(&mut self, subscriber: &str, message_handler: &dyn Fn(QueueMessage)) {
        unimplemented!()
    }

    fn unsubscribe(&mut self, subscriber: &str) {
        unimplemented!()
    }
}

#[test]
fn topic_tdd() {
    let queue_worker : RefCell<Box<dyn QueueWorker>> = RefCell::new(Box::new(MemQueueWorker::new()));
    let topic_worker = RefCell::new(MemTopicWorker::new(queue_worker));


    let sub = topic_worker.borrow_mut().subscribe("test").unwrap();
    println!("{}", sub)
}
