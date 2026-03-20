#![allow(clippy::new_without_default)]

pub mod args;
pub mod kbd_event;
pub mod kbd_event_listener;
pub mod keycode;
pub mod plugins;
pub mod publisher;
pub mod time;

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));

pub const STATE_DIR: &str = concat!("/var/lib/", env!("CARGO_PKG_NAME"), "/");

#[derive(Copy, Clone, Debug)]
pub enum Event {
    KeyEvent(kbd_event::KbdEvent),
    PluginStop,
}

impl From<kbd_event::KbdEvent> for Event {
    fn from(value: kbd_event::KbdEvent) -> Self {
        Self::KeyEvent(value)
    }
}
