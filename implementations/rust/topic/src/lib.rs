pub mod topic {
    use std::collections::HashMap;

    trait Subscription {
        fn has_messages(&self) -> bool {
            false
        }
        
        fn on_message(&mut self, message_handler: Box<dyn Fn() -> ()>);

        fn poll(&mut self);
    }

    struct MemSubscription {
        message_handler: Box<dyn Fn() -> ()>
    }

    impl MemSubscription {
        fn new() -> MemSubscription {
            MemSubscription {
                message_handler: Box::new(|| {
                    unimplemented!()
                })
            }
        }
    }

    impl Subscription for MemSubscription {
        fn on_message(&mut self, message_handler: Box<dyn Fn() -> ()>) {
            self.message_handler = message_handler
        }

        fn poll(&mut self) {
            (self.message_handler)()
        }
    }

    trait Topic {
        fn subscribe(&self) -> Box<dyn Subscription>;
    }

    struct MemTopic {

    }

    impl MemTopic {

    }

    impl Topic for MemTopic {
        fn subscribe(&self) -> Box<dyn Subscription> {
            Box::new(MemSubscription::new())
        }
    }

    trait TopicManager {
        fn get_topic(&self, name: &str) -> &Box<dyn Topic>;
    }

    struct LocalTopicManager<'a> {
        topics: HashMap<&'a str, Box<dyn Topic>>
    }

    impl LocalTopicManager<'_> {
        fn new<'a>() -> LocalTopicManager<'a>  {
            let mut topics = HashMap::new();
            let dummy : Box<dyn Topic> = Box::new(MemTopic {});

            topics.insert("dummy", dummy);
            LocalTopicManager {
                topics
            }
        }
    }

    impl TopicManager for LocalTopicManager<'_> {
        fn get_topic(&self, name: &str) -> &Box<dyn Topic> {
            self.topics.get(name).expect(format!("Topic not found {}", name).as_str())
        }
    }

    #[test]
    fn topic_tdd() {
        let topic_manager = LocalTopicManager::new();
        let topic = topic_manager.get_topic("dummy");

        let mut subscription = topic.subscribe();
        assert_eq!(false, subscription.has_messages());

        subscription.on_message(Box::new(|| {
            println!("Called")
        }));

        subscription.poll();
    }

}