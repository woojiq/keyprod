#![allow(clippy::new_without_default)]

pub mod args;
pub mod kbd_event;
pub mod kbd_event_listener;
pub mod keycode;
pub mod publisher;
pub mod subscribers;
pub mod time;

#[macro_export]
// https://github.com/rharish101/ReGreet/blob/a011d5d557c11e7a7d63eaa6cf061618721c81bc/src/constants.rs#L9
/// Get an environment variable during compile time, else return a default.
macro_rules! env_or {
    ($name:expr, $default:expr) => {
        // This is needed because `Option.unwrap_or` is not a const fn:
        // https://github.com/rust-lang/rust/issues/91930
        if let Some(value) = option_env!($name) {
            value
        } else {
            $default
        }
    };
}
