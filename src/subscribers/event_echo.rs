use super::KeyboardEventSubscriber;

pub struct EventEcho {
    logic: EventEchoLogic,
}

struct EventEchoLogic {
    writer: Box<dyn std::io::Write + Send>,
}

impl EventEcho {
    pub fn new() -> Self {
        let stdout = std::io::stdout();
        Self {
            logic: EventEchoLogic::new(Box::new(stdout)),
        }
    }
}

impl KeyboardEventSubscriber for EventEcho {
    fn event_cb(&mut self, event: crate::kbd_event::KbdEvent) {
        self.logic.register_keypress(event);
    }

    fn describe(&self) -> &'static str {
        "EventEchoLogic"
    }
}

impl EventEchoLogic {
    pub fn new(writer: Box<dyn std::io::Write + Send>) -> Self {
        Self { writer }
    }

    fn register_keypress(&mut self, event: crate::kbd_event::KbdEvent) {
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
}
