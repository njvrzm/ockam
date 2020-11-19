pub mod topic {
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::cell::RefCell;
    use std::borrow::Borrow;

    #[derive(Copy, Clone)]
    struct TopicMessage<'a> {
        body: &'a[u8]
    }

    trait Subscription {
        fn on_message(&mut self, message_handler: Box<dyn Fn(&TopicMessage) -> ()>);

        fn handler(&self) -> &Box<dyn Fn(&TopicMessage) -> ()>;
    }

    struct MemSubscription {
        message_handler: Box<dyn Fn(&TopicMessage) -> ()>
    }

    impl MemSubscription {
        fn new() -> MemSubscription {
            MemSubscription {
                message_handler: Box::new(|_message| {
                    unimplemented!()
                })
            }
        }
    }

    impl Subscription for MemSubscription {
        fn on_message(&mut self, message_handler: Box<dyn Fn(&TopicMessage) -> ()>) {
            self.message_handler = message_handler
        }

        fn handler(&self) -> &Box<dyn Fn(&TopicMessage)> {
            &self.message_handler
        }
    }

    trait Topic {
    }

    struct MemTopic {

    }

    impl MemTopic {

    }

    impl Topic for MemTopic {

    }

    trait TopicManager {
        fn get_topic(&self, topic_name: &str) -> &Box<dyn Topic>;

        fn publish(&mut self, topic_name: &str, message: TopicMessage);
    }

    struct LocalTopicManager<'a> {
        topics: HashMap<&'a str, Box<dyn Topic>>,
        subscriptions: Vec<Rc<RefCell<dyn Subscription>>>
    }

    impl LocalTopicManager<'_> {
        fn new<'a>() -> LocalTopicManager<'a>  {
            let mut topics = HashMap::new();
            let dummy : Box<dyn Topic> = Box::new(MemTopic {});

            topics.insert("dummy", dummy);
            LocalTopicManager {
                topics,
                subscriptions: Vec::new()
            }
        }

        fn subscribe(&mut self, name: &str, message_handler: Box<dyn Fn(&TopicMessage) -> ()>) -> usize {
            let subscription: Rc<RefCell<dyn Subscription>> = Rc::new(RefCell::new(MemSubscription::new()));

            subscription.borrow_mut().on_message(message_handler);

            self.subscriptions.push(subscription);
            self.subscriptions.len()
        }
    }

    impl TopicManager for LocalTopicManager<'_> {
        fn get_topic(&self, topic_name: &str) -> &Box<dyn Topic> {
            self.topics.get(topic_name).expect(format!("Topic not found {}", topic_name).as_str())
        }

        fn publish(&mut self, name: &str, message: TopicMessage) {
            for subscription in &self.subscriptions {
                (*subscription.borrow_mut()).handler()(&message);
            }
        }
    }

    #[test]
    fn topic_tdd() {
        let mut topic_manager = LocalTopicManager::new();
        let _subscription_handle = topic_manager.subscribe("dummy", Box::new(|&_message| {
            println!("Message contains {} bytes", _message.body.len())
        }));

        let body = [1, 2, 3];

        let message = TopicMessage {
            body: &(body)
        };

        topic_manager.publish("dummy", message);
    }

}