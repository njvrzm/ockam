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

    impl MemTopic<'_>  {

    }

    impl Topic for MemTopic<'_> {
        fn name(&self) -> &str {
            &self.name
        }
    }

    struct MemSubscription {
        message_handler: Box<dyn Fn(&TopicMessage) -> ()>,
        topic: Rc<Box<dyn Topic>>
    }

    impl MemSubscription {

    }

    trait Subscription {
        fn on_message(&mut self, message_handler: Box<dyn Fn(&TopicMessage) -> ()>);

        fn handler(&self) -> &Box<dyn Fn(&TopicMessage) -> ()>;

        fn topic(&self) -> Rc<Box<dyn Topic>>;
    }

    impl Subscription for MemSubscription {
        fn on_message(&mut self, message_handler: Box<dyn Fn(&TopicMessage) -> ()>) {
            self.message_handler = message_handler
        }

        fn handler(&self) -> &Box<dyn Fn(&TopicMessage)> {
            &self.message_handler
        }

        fn topic(&self) -> Rc<Box<dyn Topic>> {
            self.topic.clone()
        }
    }

    trait TopicManager {
        fn create_topic(&mut self, topic_name: &'static str);

        fn get_topic(&self, topic_name: &str) -> Option<&Rc<Box<dyn Topic>>>;

        fn publish(&mut self, topic_name: &str, message: TopicMessage);

        fn subscribe(&mut self, name: &str, message_handler: Box<dyn Fn(&TopicMessage) -> ()>) -> Option<usize>;
    }

    struct LocalTopicManager<'a> {
        topics: HashMap<&'a str, Rc<Box<dyn Topic>>>,
        subscriptions: Vec<Rc<RefCell<dyn Subscription>>>
    }

    impl LocalTopicManager<'_> {
        fn new<'a>() -> LocalTopicManager<'a>  {
            LocalTopicManager {
                topics:  HashMap::new(),
                subscriptions: Vec::new()
            }
        }
    }

    impl TopicManager for LocalTopicManager<'_> {
        fn create_topic(&mut self, topic_name: &'static str) {
            let topic : Rc<Box<dyn Topic>> = Rc::new(Box::new(MemTopic {name: topic_name}));
            self.topics.insert(topic_name, topic);
        }

        fn get_topic(&self, topic_name: &str) -> Option<&Rc<Box<dyn Topic>>> {
            self.topics.get(topic_name)
        }

        fn publish(&mut self, topic_name: &str, message: TopicMessage) {
            for subscription in &self.subscriptions {
                if (*subscription.borrow()).topic().name().eq(topic_name) {
                    (*subscription.borrow_mut()).handler()(&message);
                }
            }
        }

        fn subscribe(&mut self, name: &str, message_handler: Box<dyn Fn(&TopicMessage) -> ()>) -> Option<usize> {
            let maybe_topic = self.get_topic(name);

            match maybe_topic {
                Some(topic) => {
                    let subscription: Rc<RefCell<dyn Subscription>> = Rc::new(RefCell::new(MemSubscription {
                        message_handler,
                        topic: (*topic).clone()
                    }));
                    self.subscriptions.push(subscription);
                    Some(self.subscriptions.len() - 1)
                },
                None => None
            }
        }
    }

    #[test]
    fn topic_tdd() {
        let mut topic_manager = LocalTopicManager::new();

        topic_manager.create_topic("topic1");
        topic_manager.create_topic("topic2");

        let _subscription_handle = topic_manager.subscribe("topic1", Box::new(|&_message| {
            println!("Handler 1: Message contains {} bytes", _message.body.len())
        })).unwrap();

        let body = [1, 2, 3];

        let message = TopicMessage {
            body: &(body)
        };

        topic_manager.publish("topic1", message);
        topic_manager.publish("topic2", message);

    }

}