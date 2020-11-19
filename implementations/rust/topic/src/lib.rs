pub mod topic {
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::cell::RefCell;

    #[derive(Copy, Clone)]
    struct TopicMessage<'a> {
        body: &'a[u8]
    }

    trait Topic {
        fn name(&self) -> &str;
    }

    struct MemTopic<'a> {
        name: &'a str
    }

    impl Topic for MemTopic<'_> {
        fn name(&self) -> &str {
            &self.name
        }
    }

    type MessageHandler = Box<dyn Fn(&TopicMessage) -> ()>;
    type TopicHandle = Rc<Box<dyn Topic>>;

    struct MemSubscription {
        message_handler: MessageHandler,
        topic: TopicHandle,
        id: usize
    }

    trait Subscription {
        fn on_message(&mut self, message_handler: MessageHandler);

        fn handler(&self) -> &MessageHandler;

        fn topic(&self) -> TopicHandle;

        fn id(&self) -> usize;
    }

    type SubscriptionHandle = Rc<RefCell<dyn Subscription>>;

    impl Subscription for MemSubscription {
        fn on_message(&mut self, message_handler: MessageHandler) {
            self.message_handler = message_handler
        }

        fn handler(&self) -> &MessageHandler {
            &self.message_handler
        }

        fn topic(&self) -> Rc<Box<dyn Topic>> {
            self.topic.clone()
        }

        fn id(&self) -> usize {
            self.id
        }
    }


    trait TopicManager {
        fn create_topic(&mut self, topic_name: &'static str);

        fn get_topic(&self, topic_name: &str) -> Option<&TopicHandle>;

        fn publish(&mut self, topic_name: &str, message: TopicMessage);

        fn subscribe(&mut self, name: &str, message_handler: MessageHandler) -> Option<SubscriptionHandle>;

        fn unsubscribe(&mut self, handle: SubscriptionHandle);
    }

    struct LocalTopicManager<'a> {
        topics: HashMap<&'a str, TopicHandle>,
        subscriptions: Vec<SubscriptionHandle>,
        subscription_id_counter: usize
    }

    impl LocalTopicManager<'_> {
        fn new<'a>() -> LocalTopicManager<'a>  {
            LocalTopicManager {
                topics:  HashMap::new(),
                subscriptions: Vec::new(),
                subscription_id_counter: 0
            }
        }
    }

    impl TopicManager for LocalTopicManager<'_> {
        fn create_topic(&mut self, topic_name: &'static str) {
            let topic : Rc<Box<dyn Topic>> = Rc::new(Box::new(MemTopic {name: topic_name}));
            self.topics.insert(topic_name, topic);
        }

        fn get_topic(&self, topic_name: &str) -> Option<&TopicHandle> {
            self.topics.get(topic_name)
        }

        fn publish(&mut self, topic_name: &str, message: TopicMessage) {
            for subscription in &self.subscriptions {
                if (*subscription.borrow()).topic().name().eq(topic_name) {
                    (*subscription.borrow_mut()).handler()(&message);
                }
            }
        }

        fn subscribe(&mut self, name: &str, message_handler: MessageHandler) -> Option<SubscriptionHandle> {
            let maybe_topic = self.get_topic(name);

            match maybe_topic {
                Some(topic) => {
                    let subscription: SubscriptionHandle = Rc::new(RefCell::new(MemSubscription {
                        message_handler,
                        topic: (*topic).clone(),
                        id: self.subscription_id_counter
                    }));
                    self.subscriptions.push(subscription.clone());
                    self.subscription_id_counter += 1;
                    Some(subscription.clone())
                },
                None => None
            }
        }

        fn unsubscribe(&mut self, handle: SubscriptionHandle) {
            self.subscriptions.retain(|s| s.borrow().id() != handle.borrow().id())
        }
    }

    #[test]
    fn topic_tdd() {
        let mut topic_manager = LocalTopicManager::new();

        topic_manager.create_topic("topic1");
        topic_manager.create_topic("topic2");

        let subscription1 = topic_manager.subscribe("topic1", Box::new(|&_message| {
            println!("Handler 1: Message contains {} bytes", _message.body.len())
        })).unwrap();

        let subscription2 = topic_manager.subscribe("topic2", Box::new(|&_message| {
            println!("Handler 2: Bytes {:?} ", _message.body)
        })).unwrap();

        let body = [1, 2, 3];

        let message = TopicMessage {
            body: &(body)
        };

        topic_manager.publish("topic1", message);
        topic_manager.publish("topic2", message);

        topic_manager.unsubscribe(subscription1);
        topic_manager.unsubscribe(subscription2);
    }

}