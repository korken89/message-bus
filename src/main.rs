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
    use std::sync::Arc;
    use tokio::sync::broadcast::{channel, error::RecvError, Receiver, Sender};

    pub use sub_topic::SubTopic;

    pub mod sub_topic {
        use super::*;

        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static TOPIC_SubTopic: Lazy<Sender<SubTopic>> = Lazy::new(|| channel(1).0);

        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static TOPIC_SubTopic_Foo: Lazy<Sender<Arc<u32>>> = Lazy::new(|| channel(1).0);

        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static TOPIC_SubTopic_Bar: Lazy<Sender<Arc<u8>>> = Lazy::new(|| channel(1).0);

        /// Receiver for sub-topics `Foo`, `Bar`.
        #[derive(Clone)]
        pub enum SubTopic {
            /// Receiver for `Foo`.
            Foo(Arc<u32>),
            /// Receiver for `Bar`.
            Bar(Arc<u8>),
        }

        impl SubTopic {
            /// Subscribe to all topics under `SubTopic`.
            pub fn subscribe() -> Receiver<SubTopic> {
                TOPIC_SubTopic.subscribe()
            }
        }

        fn publish(payload: SubTopic) {
            TOPIC_SubTopic.send(payload).ok();
        }

        pub struct Foo {}

        impl Foo {
            pub fn subscribe() -> Receiver<Arc<u32>> {
                TOPIC_SubTopic_Foo.subscribe()
            }

            pub fn publish(payload: u32) {
                let payload = Arc::new(payload);

                TOPIC_SubTopic_Foo.send(payload.clone()).ok();

                publish(SubTopic::Foo(payload))
            }
        }

        pub struct Bar {}

        impl Bar {
            pub fn subscribe() -> Receiver<Arc<u8>> {
                TOPIC_SubTopic_Bar.subscribe()
            }

            pub fn publish(payload: u8) {
                let payload = Arc::new(payload);

                TOPIC_SubTopic_Bar.send(payload.clone()).ok();

                publish(SubTopic::Bar(payload))
            }
        }
    }

    /// Handle to the `SystemHealth` topic on the bus.
    pub struct SystemHealth {}

    #[doc(hidden)]
    #[allow(non_upper_case_globals)]
    static TOPIC_SystemHealth: Lazy<Sender<Arc<String>>> = Lazy::new(|| channel(1).0);

    impl SystemHealth {
        pub fn subscribe() -> Receiver<Arc<String>> {
            TOPIC_SystemHealth.subscribe()
        }

        pub fn publish(payload: String) {
            TOPIC_SystemHealth.send(Arc::new(payload)).ok();
        }
    }

    /// Handle to the `SomeData` topic on the bus.
    pub struct SomeData {}

    #[doc(hidden)]
    #[allow(non_upper_case_globals)]
    static TOPIC_SomeData: Lazy<Sender<Arc<u32>>> = Lazy::new(|| channel(1).0);

    impl SomeData {
        pub fn subscribe() -> Receiver<Arc<u32>> {
            TOPIC_SomeData.subscribe()
        }

        pub fn publish(payload: u32) {
            TOPIC_SomeData.send(Arc::new(payload)).ok();
        }
    }

    //
    // ----- end macro expansion -----
    //

    /// Will flush errors (`Lagged`), and finaly give out the latest value.
    ///
    /// The closue `on_err` will run for all errors, and the `Closed` error will never come as the
    /// producer is a static variable that is never dropped.
    pub async fn recv_with_flush<T: Clone, F: FnMut(RecvError)>(
        receiver: &mut Receiver<T>,
        mut on_err: F,
    ) -> T {
        loop {
            match receiver.recv().await {
                Ok(msg) => break msg,
                Err(e) => {
                    // The oldest value has been overwritten here, recv again to get the "new"
                    // oldest value.
                    on_err(e);
                }
            }
        }
    }

    /// Receive and ignore `Lagged` errors. This will provide the latest value available.
    pub async fn recv_ignore_error<T: Clone>(receiver: &mut Receiver<T>) -> T {
        recv_with_flush(receiver, |_| {}).await
    }
}

use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    tokio::spawn(async {
        let mut sub1 = message_bus::SomeData::subscribe();

        loop {
            let msg = message_bus::recv_ignore_error(&mut sub1).await;
            println!("Got {msg:?} on task 1");
        }
    });

    tokio::spawn(async {
        let mut sub1 = message_bus::SomeData::subscribe();

        loop {
            let msg = message_bus::recv_ignore_error(&mut sub1).await;
            println!("Got {msg} on task 2");

            sleep(Duration::from_millis(150)).await;
        }
    });

    // v.push(tokio::spawn(async {
    //     let mut sub1 = message_bus::SomeData::subscribe();

    //     loop {
    //         let msg = sub1.recv().await;
    //         println!("Got {msg:?} on task 3");

    //         sleep(Duration::from_millis(1_000)).await;
    //     }
    // }));

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
