pub mod event_echo;
pub mod event_history;

pub use event_echo::EventEcho;
pub use event_history::EventHistory;

pub trait KeyboardEventSubscriber {
    fn event_cb(&mut self, event: crate::kbd_event::KbdEvent);

    fn describe(&self) -> &'static str;
}
