use super::KeyboardEventSubscriber;

pub struct EventEcho {
    writer: Box<dyn std::io::Write + Send>,
}

impl EventEcho {
    pub fn new(writer: Box<dyn std::io::Write + Send>) -> Self {
        Self { writer }
    }

    pub fn new_stdout() -> Self {
        let stdout = std::io::stdout();
        Self::new(Box::new(stdout) as Box<dyn std::io::Write + Send>)
    }
}

impl KeyboardEventSubscriber for EventEcho {
    fn event_cb(&mut self, event: crate::kbd_event::KbdEvent) {
        if event.is_press() || event.is_release() {
            writeln!(
                self.writer,
                "{}({}) was {}",
                event.code,
                event.code as u16,
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
