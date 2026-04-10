use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::app::event::AppEvent;

pub struct EventBus {
    sender: Sender<AppEvent>,
    receiver: Receiver<AppEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }

    pub fn sender(&self) -> Sender<AppEvent> {
        self.sender.clone()
    }

    pub fn receiver(&self) -> &Receiver<AppEvent> {
        &self.receiver
    }
}
