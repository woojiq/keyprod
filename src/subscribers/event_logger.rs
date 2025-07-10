use crate::input_event_codes::KEY_TO_STR_REPR;

use super::KeyboardEventSubscriber;

pub struct EventLogger {
    writer: Box<dyn std::io::Write + Send>,
}

impl EventLogger {
    pub fn new(writer: Box<dyn std::io::Write + Send>) -> Self {
        Self { writer }
    }

    pub fn new_stdout() -> Self {
        let stdout = std::io::stdout();
        Self::new(Box::new(stdout) as Box<dyn std::io::Write + Send>)
    }
}

impl KeyboardEventSubscriber for EventLogger {
    fn event_cb(&mut self, event: crate::kbd_event::KbdEvent) {
        if event.is_press() || event.is_release() {
            writeln!(
                self.writer,
                "{}({}) was {}",
                KEY_TO_STR_REPR[event.code as usize],
                event.code,
                if event.is_press() {
                    "Pressed"
                } else {
                    "Released"
                },
            )
            .unwrap();
        }
    }

    fn describe(&self) -> &'static str {
        "EventLogger"
    }
}
