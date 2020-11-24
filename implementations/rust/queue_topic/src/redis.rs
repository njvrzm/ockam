use crate::topic::*;
use simple_redis::client::Client;
use simple_redis::{Interrupts, Message};
use std::cell::RefCell;
use std::io;
use std::io::Write;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

struct RedisWorker {
    client: Client,
    subscription_counter: usize,
}

impl RedisWorker {
    fn new(url: &str) -> Option<RedisWorker> {
        match simple_redis::create(url) {
            Ok(client) => Some(RedisWorker {
                client,
                subscription_counter: 0,
            }),
            Err(_) => None,
        }
    }
}

impl TopicWorker for RedisWorker {
    fn publish(&mut self, topic_name: String, message: TopicMessage) {
        unimplemented!()
    }

    fn subscribe(&mut self, name: String) -> Option<SubscriptionHandle> {
        self.client.subscribe(name.as_str());

        self.subscription_counter += 1;

        let topic = MemTopic::new(name);
        Some(Rc::new(RefCell::new(TopicSubscription {
            topic,
            id: self.subscription_counter,
        })))
    }

    fn poll(&mut self, handle: SubscriptionHandle, message_handler: &dyn Fn(TopicMessage)) {
        self.client.fetch_messages(
            &mut |message: Message| -> bool {
                let topic = (*handle).borrow().topic();
                let name = (*topic).name();
                if name.as_str() == message.get_channel_name() {
                    let payload: String = message.get_payload().unwrap();
                    message_handler(TopicMessage {
                        body: payload.as_bytes(),
                    });
                }
                false
            },
            &mut || -> Interrupts {
                let mut int = Interrupts::new();
                int.next_polling_time = Some(100);
                int
            },
        );
    }

    fn unsubscribe(&mut self, handle: SubscriptionHandle) {
        let sub = (*handle).borrow();
        self.client.unsubscribe(&(*sub).topic().name());
    }
}

#[test]
fn redis_tdd() -> () {

    let mut manager = RedisWorker::new("redis://127.0.0.1:6379/").unwrap();

    let sub = manager.subscribe("test".to_string()).unwrap();


    manager.poll(sub.clone(), &|message| {
        println!("Message: {:?}", message.body);
    });

    manager.unsubscribe(sub.clone());
    println!("hello2")
}
