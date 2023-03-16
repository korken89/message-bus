//! Crate

#![deny(missing_docs)]

/// The message bus definition rescides here.
pub mod message_bus {
    // make_message_bus!(
    //     sub_topic::SubTopic => {
    //        Foo => u32,
    //        Bar => u8,
    //     },
    //     SystemHealth => String,
    //     SomeData => u32,
    // )

    //
    // ----- macro expansion -----
    //
    use once_cell::sync::Lazy;
    use tokio::sync::broadcast::{
        channel,
        error::{RecvError, TryRecvError},
        Receiver, Sender,
    };

    /// A subscriber to a topic on the bus.
    pub struct Subscriber<T: Clone>(Receiver<T>);

    impl<T> Subscriber<T>
    where
        T: Clone,
    {
        /// Tries to receive a value, will return `None` if there are none.
        pub fn try_recv(&mut self) -> Option<T> {
            loop {
                match self.0.try_recv() {
                    Ok(v) => return Some(v),
                    Err(TryRecvError::Empty) => return None,
                    Err(TryRecvError::Lagged(_)) => {} // Skip lagged errors
                    Err(TryRecvError::Closed) => unreachable!(), // Impossible to drop the sender
                }
            }
        }

        /// Receive a value from the bus.
        pub async fn recv(&mut self) -> T {
            loop {
                match self.0.recv().await {
                    Ok(msg) => return msg,
                    Err(RecvError::Lagged(_)) => {} // Skip lagged errors
                    Err(RecvError::Closed) => unreachable!(), // Impossible to drop the sender
                }
            }
        }

        /// Checks if there is a message on the topic.
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
    }

    pub use sub_topic::SubTopic;

    #[allow(missing_docs)]
    pub mod sub_topic {
        use super::*;

        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static TOPIC_SubTopic: Lazy<Sender<SubTopic>> = Lazy::new(|| channel(1).0);

        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static TOPIC_SubTopic_Foo: Lazy<Sender<u32>> = Lazy::new(|| channel(1).0);

        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static TOPIC_SubTopic_Bar: Lazy<Sender<u8>> = Lazy::new(|| channel(1).0);

        /// Receiver for sub-topics `Foo`, `Bar`.
        #[derive(Clone)]
        pub enum SubTopic {
            /// Receiver for `Foo`.
            Foo(u32),
            /// Receiver for `Bar`.
            Bar(u8),
        }

        impl SubTopic {
            /// Subscribe to all topics under `SubTopic`.
            pub fn subscribe() -> Subscriber<SubTopic> {
                Subscriber(TOPIC_SubTopic.subscribe())
            }
        }

        /// Publish to `SubTopic`.
        pub fn publish(payload: SubTopic) {
            TOPIC_SubTopic.send(payload).ok();
        }

        /// Handle to the `Foo` topic.
        pub struct Foo {}

        impl Foo {
            /// Subscribe to the `Foo` topic.
            pub fn subscribe() -> Subscriber<u32> {
                Subscriber(TOPIC_SubTopic_Foo.subscribe())
            }

            /// Publish to the `Foo` topic.
            pub fn publish(payload: u32) {
                TOPIC_SubTopic_Foo.send(payload.clone()).ok();

                publish(SubTopic::Foo(payload))
            }
        }

        /// Handle to the `Bar` topic.
        pub struct Bar {}

        impl Bar {
            /// Subscribe to the `Bar` topic.
            pub fn subscribe() -> Subscriber<u8> {
                Subscriber(TOPIC_SubTopic_Bar.subscribe())
            }

            /// Publish to the `Bar` topic.
            pub fn publish(payload: u8) {
                TOPIC_SubTopic_Bar.send(payload.clone()).ok();

                publish(SubTopic::Bar(payload))
            }
        }
    }

    /// Handle to the `SystemHealth` topic.
    pub struct SystemHealth {}

    #[doc(hidden)]
    #[allow(non_upper_case_globals)]
    static TOPIC_SystemHealth: Lazy<Sender<String>> = Lazy::new(|| channel(1).0);

    impl SystemHealth {
        /// Subscribe to the `SystemHealth` topic.
        pub fn subscribe() -> Subscriber<String> {
            Subscriber(TOPIC_SystemHealth.subscribe())
        }

        /// Publish to the `SystemHealth` topic.
        pub fn publish(payload: String) {
            TOPIC_SystemHealth.send(payload).ok();
        }
    }

    /// Handle to the `SomeData` topic.
    pub struct SomeData {}

    #[doc(hidden)]
    #[allow(non_upper_case_globals)]
    static TOPIC_SomeData: Lazy<Sender<u32>> = Lazy::new(|| channel(1).0);

    impl SomeData {
        /// Subscribe to the `SomeData` topic.
        pub fn subscribe() -> Subscriber<u32> {
            Subscriber(TOPIC_SomeData.subscribe())
        }

        /// Publish to the `SomeData` topic.
        pub fn publish(payload: u32) {
            TOPIC_SomeData.send(payload).ok();
        }
    }

    //
    // ----- end macro expansion -----
    //
}

use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    tokio::spawn(async {
        let mut sub1 = message_bus::SomeData::subscribe();

        loop {
            let msg = sub1.recv().await;
            println!("Got {msg:?} on task 1");
        }
    });

    tokio::spawn(async {
        let mut sub1 = message_bus::SomeData::subscribe();

        loop {
            let msg = sub1.recv().await;
            println!("Got {msg} on task 2");

            sleep(Duration::from_millis(150)).await;
        }
    });

    tokio::spawn(async {
        let mut sub1 = message_bus::SomeData::subscribe();

        loop {
            let msg = sub1.recv().await;
            println!("Got {msg:?} on task 3");

            sleep(Duration::from_millis(1_000)).await;
        }
    });

    tokio::spawn(async {
        for i in 0..100 {
            // println!("Sending {i} from task 4");
            message_bus::SomeData::publish(i);

            sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .ok();

    println!("Hello, world!");
}
