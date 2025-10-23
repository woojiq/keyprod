pub mod event_echo;
pub mod event_history;

pub use event_echo::EventEcho;
pub use event_history::EventHistory;

pub trait KeyboardEventSubscriber {
    fn event_cb(&mut self, event: crate::kbd_event::KbdEvent);

    fn describe(&self) -> &'static str;
}

pub struct KeyboardEventSubscriberFactory {}

impl KeyboardEventSubscriberFactory {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create_subscriber(name: &str) -> Option<Box<dyn KeyboardEventSubscriber>> {
        match name {
            "echo" => Some(Box::new(EventEcho::new())),
            "history" => Some(Box::new(EventHistory::new())),
            _ => {
                eprintln!("Subscriber with name '{name}' doesn't exist.");
                None
            }
        }
    }

    pub fn create_subscribers<T: AsRef<str>>(names: &[T]) -> Vec<Box<dyn KeyboardEventSubscriber>> {
        names
            .iter()
            .filter_map(|s| Self::create_subscriber(s.as_ref()))
            .collect()
    }
}
