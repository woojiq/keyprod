pub mod event_echo;

pub use event_echo::EventEcho;

pub trait KeyboardEventSubscriber: Send {
    fn event_cb(&mut self, event: crate::kbd_event::KbdEvent);

    fn describe(&self) -> &'static str;
}
