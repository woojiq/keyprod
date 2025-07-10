use crate::{kbd_event::KbdEvent, subscribers::KeyboardEventSubscriber};

pub trait KeyboardEventPublisher {
    fn run(&mut self);

    fn register_subscriber(&mut self, subscriber: Box<dyn KeyboardEventSubscriber>);
}

pub struct DefaultKeyboardEventPublisher {
    receiver: std::sync::mpsc::Receiver<KbdEvent>,
    subscribers: Vec<Box<dyn KeyboardEventSubscriber>>,
}

impl DefaultKeyboardEventPublisher {
    pub fn new(receiver: std::sync::mpsc::Receiver<KbdEvent>) -> Self {
        Self {
            receiver,
            subscribers: vec![],
        }
    }
}

impl KeyboardEventPublisher for DefaultKeyboardEventPublisher {
    fn run(&mut self) {
        println!("Publisher is ready to receive events.");

        while let Ok(event) = self.receiver.recv() {
            for subscriber in &mut self.subscribers {
                subscriber.event_cb(event);
            }
        }

        println!("Publisher finished its loop.");
    }

    fn register_subscriber(&mut self, subscriber: Box<dyn KeyboardEventSubscriber>) {
        println!(
            "Publisher accepted new subscriber: '{}'.",
            subscriber.describe(),
        );

        self.subscribers.push(subscriber);

        println!("Current number of subscribers: {}.", self.subscribers.len());
    }
}
