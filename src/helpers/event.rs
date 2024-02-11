use crossbeam_channel::{unbounded, Receiver, Sender};

pub struct Event<T> {
    senders: Vec<Sender<T>>,
}

impl<T: Clone> Event<T> {
    pub fn new() -> Self {
        Event {
            senders: Vec::new(),
        }
    }

    pub fn subscribe(&mut self) -> Receiver<T> {
        let (sender, receiver) = unbounded();
        self.senders.push(sender);
        receiver
    }

    pub fn broadcast(&mut self, message: T) {
        self.senders
            .retain(|sender| sender.send(message.clone()).is_ok());
    }
}
