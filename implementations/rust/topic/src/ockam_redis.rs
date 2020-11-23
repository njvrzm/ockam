pub mod ockam_redis {
    use crate::topic::*;
    use simple_redis::{Message, Interrupts};
    use simple_redis::client::Client;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::borrow::Borrow;

    struct RedisManager {
        client: Client,
        subscription_counter: usize
    }

    impl RedisManager {
        fn new(url: &str) -> Option<RedisManager> {
            match simple_redis::create(url) {
                Ok(client) => Some(RedisManager { client, subscription_counter: 0 }),
                Err(_) => None
            }
        }
    }

    impl TopicManager for RedisManager {
        fn publish(&mut self, topic_name: String, message: TopicMessage) {
            unimplemented!()
        }

        fn subscribe(&mut self, name: String) -> Option<SubscriptionHandle> {
            self.client.subscribe(name.as_str());

            self.subscription_counter += 1;

            let topic = MemTopic::new(name);
            Some(Rc::new(RefCell::new(TopicSubscription {
                topic,
                id: self.subscription_counter
            })))    
        }

        fn poll(&mut self, handle: SubscriptionHandle, message_handler: &dyn Fn(TopicMessage)) {
            self.client.fetch_messages(&mut |message: Message| -> bool {
                let topic = (*handle).borrow().topic();
                let name = (*topic).name();
                if name.as_str() == message.get_channel_name() {
                    let payload: String = message.get_payload().unwrap();
                    message_handler(TopicMessage {
                        body: payload.as_bytes()
                    });
                }
                true
            }, &mut || -> Interrupts {
                Interrupts::new()
            });
        }

        fn unsubscribe(&mut self, handle: SubscriptionHandle) {
            let sub = (*handle).borrow();
            self.client.unsubscribe(&(*sub).topic().name());
        }
    }

    #[test]
    fn redis_tdd() -> () {
        let mut manager = RedisManager::new("redis://127.0.0.1:6379/").unwrap();

        let sub = manager.subscribe("test".to_string()).unwrap();

        manager.poll(sub.clone(), &|message| {
            println!("Message: {:?}", message.body)
        });

        manager.unsubscribe(sub.clone());
      /*  let mut client = simple_redis::create("redis://127.0.0.1:6379/").unwrap();
        let mut _result = client.subscribe("test");

        */
    }
}
