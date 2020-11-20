pub mod topic {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    #[derive(Copy, Clone)]
    pub struct TopicMessage<'a> {
        body: &'a [u8],
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
        pub message_handler: MessageHandler,
        pub topic: TopicHandle,
        pub id: usize,
    }


    pub trait Subscription {
        fn on_message(&mut self, message_handler: MessageHandler);

        fn message_handler(&self) -> &MessageHandler;

        fn topic(&self) -> TopicHandle;

        fn id(&self) -> usize;
    }

    pub(crate) type SubscriptionHandle = Rc<RefCell<dyn Subscription>>;

    impl Subscription for TopicSubscription {
        fn on_message(&mut self, message_handler: MessageHandler) {
            self.message_handler = message_handler
        }

        fn message_handler(&self) -> &MessageHandler {
            &self.message_handler
        }

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

    pub trait TopicManager {
        fn publish(&mut self, topic_name: String, message: TopicMessage);

        fn subscribe(
            &mut self,
            name: String,
            message_handler: MessageHandler,
        ) -> Option<SubscriptionHandle>;

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
            let topic: Rc<Box<dyn Topic>> = Rc::new(Box::new(MemTopic { topic_name: topic_name.clone() }));
            self.topics.insert(topic_name.clone(), topic);
        }

        fn get_topic(&self, topic_name: String) -> Option<&TopicHandle> {
            self.topics.get(topic_name.as_str())
        }
    }

    impl TopicManager for LocalTopicManager {
        fn publish(&mut self, topic_name: String, message: TopicMessage) {
            for subscription in &self.subscriptions {
                if (*subscription.borrow()).topic().name().eq(topic_name.clone().as_str()) {
                    (*subscription.borrow_mut()).message_handler()(&message);
                }
            }
        }

        fn subscribe(
            &mut self,
            topic_name: String,
            message_handler: MessageHandler,
        ) -> Option<SubscriptionHandle> {
            let maybe_topic = self.get_topic(topic_name);

            match maybe_topic {
                Some(topic) => {
                    let subscription: SubscriptionHandle = Rc::new(RefCell::new(TopicSubscription {
                        message_handler,
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

        let subscription1 = topic_manager
            .subscribe(
                "topic1".to_string(),
                Box::new(|&_message| {
                    println!("Handler 1: Message contains {} bytes", _message.body.len())
                }),
            )
            .unwrap();

        let subscription2 = topic_manager
            .subscribe(
                "topic2".to_string(),
                Box::new(|&_message| println!("Handler 2: Bytes {:?} ", _message.body)),
            )
            .unwrap();

        let body = [1, 2, 3];

        let message = TopicMessage { body: &(body) };

        topic_manager.publish("topic1".to_string(), message);
        topic_manager.publish("topic2".to_string(), message);

        topic_manager.unsubscribe(subscription1);
        topic_manager.unsubscribe(subscription2);
    }
}

pub mod ockam_redis;
