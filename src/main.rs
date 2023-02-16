pub mod message_bus {
    use log::*;
    use once_cell::sync::Lazy;
    use std::sync::Arc;
    use tokio::sync::broadcast::{channel, error::RecvError, Receiver, Sender};

    // make_message_bus!(
    //     SystemHealth => String,
    //     SomeData => u32,
    // )

    //
    // ----- macro expansion -----
    //

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
            match TOPIC_SystemHealth.send(Arc::new(payload)) {
                Ok(v) => {
                    trace!(
                        "Published message to `{}`, {v} active listeners",
                        "SystemHealth"
                    )
                }
                Err(_) => {
                    trace!(
                        "Failed to published message to `{}`, no active listeners",
                        "SystemHealth"
                    )
                }
            }
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
            match TOPIC_SomeData.send(Arc::new(payload)) {
                Ok(v) => {
                    trace!("Published message to `SomeData`, {v} active listeners")
                }
                Err(_) => {
                    trace!("Failed to published message to `SomeData`, no active listeners")
                }
            }
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
