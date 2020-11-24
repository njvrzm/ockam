use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Copy, Clone)]
pub struct TopicMessage<'a> {
    pub body: &'a [u8],
}

pub trait Topic {
    fn name(&self) -> String;
}

pub struct MemTopic {
    pub topic_name: String,
}

impl MemTopic {
    pub fn new(topic_name: String) -> TopicHandle {
        Rc::new(Box::new(MemTopic { topic_name }))
    }
}

impl Topic for MemTopic {
    fn name(&self) -> String {
        self.topic_name.clone()
    }
}

pub type MessageHandler = Box<dyn Fn(&TopicMessage) -> ()>;
pub type TopicHandle = Rc<Box<dyn Topic>>;

pub struct TopicSubscription {
    pub topic: TopicHandle,
    pub id: usize,
}

pub trait Subscription {
    fn topic(&self) -> TopicHandle;

    fn id(&self) -> usize;
}

pub(crate) type SubscriptionHandle = Rc<RefCell<dyn Subscription>>;

impl Subscription for TopicSubscription {
    fn topic(&self) -> Rc<Box<dyn Topic>> {
        self.topic.clone()
    }

    fn id(&self) -> usize {
        self.id
    }
}

pub trait TopicCreator {
    fn create_topic(&mut self, topic_name: String);

    fn get_topic(&self, topic_name: String) -> Option<&TopicHandle>;
}

pub trait TopicWorker {
    fn publish(&mut self, topic_name: String, message: TopicMessage);

    fn subscribe(&mut self, name: String) -> Option<SubscriptionHandle>;

    fn poll(&mut self, handle: SubscriptionHandle, message_handler: &dyn Fn(TopicMessage) -> ());

    fn unsubscribe(&mut self, handle: SubscriptionHandle);
}

struct LocalTopicManager {
    topics: HashMap<String, TopicHandle>,
    subscriptions: Vec<SubscriptionHandle>,
    subscription_id_counter: usize,
}

impl LocalTopicManager {
    fn new() -> LocalTopicManager {
        LocalTopicManager {
            topics: HashMap::new(),
            subscriptions: Vec::new(),
            subscription_id_counter: 0,
        }
    }
}

impl TopicCreator for LocalTopicManager {
    fn create_topic(&mut self, topic_name: String) {
        let topic: Rc<Box<dyn Topic>> = Rc::new(Box::new(MemTopic {
            topic_name: topic_name.clone(),
        }));
        self.topics.insert(topic_name.clone(), topic);
    }

    fn get_topic(&self, topic_name: String) -> Option<&TopicHandle> {
        self.topics.get(topic_name.as_str())
    }
}

impl TopicWorker for LocalTopicManager {
    fn publish(&mut self, topic_name: String, message: TopicMessage) {
        for subscription in &self.subscriptions {
            if (*subscription.borrow())
                .topic()
                .name()
                .eq(topic_name.clone().as_str())
            {
                // TODO put on internal Queue
            }
        }
    }

    fn subscribe(&mut self, topic_name: String) -> Option<SubscriptionHandle> {
        let maybe_topic = self.get_topic(topic_name);

        match maybe_topic {
            Some(topic) => {
                let subscription: SubscriptionHandle = Rc::new(RefCell::new(TopicSubscription {
                    topic: (*topic).clone(),
                    id: self.subscription_id_counter,
                }));
                self.subscriptions.push(subscription.clone());
                self.subscription_id_counter += 1;
                Some(subscription.clone())
            }
            None => None,
        }
    }

    fn poll(&mut self, handle: SubscriptionHandle, func: &dyn Fn(TopicMessage) -> ()) {
        unimplemented!()
    }

    fn unsubscribe(&mut self, handle: SubscriptionHandle) {
        self.subscriptions
            .retain(|s| s.borrow().id() != handle.borrow().id())
    }
}

#[test]
fn topic_tdd() {
    let mut topic_manager = LocalTopicManager::new();

    topic_manager.create_topic("topic1".to_string());
    topic_manager.create_topic("topic2".to_string());

    let subscription1 = topic_manager.subscribe("topic1".to_string()).unwrap();

    let subscription2 = topic_manager.subscribe("topic2".to_string()).unwrap();

    let body = [1, 2, 3];

    let message = TopicMessage { body: &(body) };

    topic_manager.publish("topic1".to_string(), message);
    topic_manager.publish("topic2".to_string(), message);

    topic_manager.unsubscribe(subscription1);
    topic_manager.unsubscribe(subscription2);
}
