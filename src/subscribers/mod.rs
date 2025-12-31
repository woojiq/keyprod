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

    pub fn create_subscriber(
        name: &str,
        cli_args: &crate::args::Args,
    ) -> Option<Box<dyn KeyboardEventSubscriber>> {
        match name {
            "echo" => Some(Box::new(EventEcho::new())),
            "history" => Some(Box::new(EventHistory::new(&cli_args.state_dir))),
            _ => {
                eprintln!("Subscriber with name '{name}' doesn't exist.");
                None
            }
        }
    }

    pub fn create_subscribers(
        cli_args: &crate::args::Args,
    ) -> Vec<Box<dyn KeyboardEventSubscriber>> {
        cli_args
            .subscribers
            .iter()
            .filter_map(|s| Self::create_subscriber(s.as_ref(), cli_args))
            .collect()
    }
}
