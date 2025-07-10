pub mod event_logger;

pub use event_logger::EventLogger;

pub trait KeyboardEventSubscriber: Send {
    fn event_cb(&mut self, event: crate::kbd_event::KbdEvent);

    fn describe(&self) -> &'static str;
}
