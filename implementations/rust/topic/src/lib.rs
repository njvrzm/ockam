pub mod topic {
    use std::collections::HashMap;

    trait Subscription {
        fn has_messages(&self) -> bool {
            false
        }
    }

    struct MemSubscription {

    }

    impl Subscription for MemSubscription {

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
            Box::new(MemSubscription {})
        }
    }

    trait TopicManager {
        fn get_topic(&self, name: &str) -> &Box<dyn Topic>;
    }

    struct LocalTopicManager {
        topics: HashMap<&'static str, Box<dyn Topic>>
    }

    impl LocalTopicManager {
        fn new() -> LocalTopicManager {
            let mut topics = HashMap::new();
            let dummy : Box<dyn Topic> = Box::new(MemTopic {});

            topics.insert("dummy", dummy);
            LocalTopicManager {
                topics
            }
        }
    }

    impl TopicManager for LocalTopicManager {
        fn get_topic(&self, name: &str) -> &Box<dyn Topic> {
            self.topics.get(name).expect(format!("Topic not found {}", name).as_str())
        }
    }

    #[test]
    fn topic_tdd() {
        let topic_manager = LocalTopicManager::new();
        let topic = topic_manager.get_topic("dummy");

        let subscription = topic.subscribe();
        assert_eq!(false, subscription.has_messages())
    }

}