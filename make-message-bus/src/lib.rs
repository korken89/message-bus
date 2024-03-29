//! Crate

#![deny(missing_docs)]

pub use message_bus_macros::make_message_bus;
use once_cell::sync::Lazy;
use tokio::sync::broadcast::{
    channel,
    error::{RecvError, TryRecvError},
    Receiver, Sender,
};

/// Topic type used in static storage in codegen.
pub struct Topic<T: Clone>(Lazy<Sender<T>>);

impl<T> Topic<T>
where
    T: Clone,
{
    /// Create a new topic.
    pub const fn new<const CAP: usize>() -> Self {
        Self(Lazy::new(|| channel(CAP).0))
    }

    /// Subscribe to the topic.
    pub fn subscribe(&self) -> Subscriber<T> {
        Subscriber {
            recv: self.0.subscribe(),
            missed_messages: 0,
        }
    }

    /// Publish to a topic.
    pub fn publish(&self, payload: T) {
        self.0.send(payload).ok();
    }
}

/// A subscriber to a topic on the bus.
pub struct Subscriber<T: Clone> {
    recv: Receiver<T>,
    missed_messages: u64,
}

impl<T> Subscriber<T>
where
    T: Clone,
{
    /// Tries to receive a value, will return `None` if there are none.
    pub fn try_recv(&mut self) -> Option<T> {
        loop {
            match self.recv.try_recv() {
                Ok(v) => return Some(v),
                Err(TryRecvError::Empty) => return None,
                Err(TryRecvError::Lagged(n)) => self.missed_messages += n,
                Err(TryRecvError::Closed) => unreachable!(), // Impossible to drop the sender
            }
        }
    }

    /// Receive a value from the bus.
    pub async fn recv(&mut self) -> T {
        loop {
            match self.recv.recv().await {
                Ok(msg) => return msg,
                Err(RecvError::Lagged(n)) => self.missed_messages += n,
                Err(RecvError::Closed) => unreachable!(), // Impossible to drop the sender
            }
        }
    }

    /// Checks if there is a message on the topic.
    pub fn is_empty(&self) -> bool {
        self.recv.is_empty()
    }

    /// Number of messages missed since last time this was called.
    pub fn messages_lost(&mut self) -> u64 {
        let n = self.missed_messages;
        self.missed_messages = 0;

        n
    }
}
