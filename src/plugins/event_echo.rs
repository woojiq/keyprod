use super::Plugin;

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

#[async_trait::async_trait]
impl Plugin for EventEcho {
    fn describe(&self) -> &'static str {
        "EventEchoLogic"
    }

    async fn run(&mut self, mut rx: tokio::sync::mpsc::Receiver<crate::Event>) {
        while let Some(event) = rx.recv().await {
            match event {
                crate::Event::KeyEvent(kbd_ev) => self.logic.register_keypress(kbd_ev),
                crate::Event::PluginStop => break,
            }
        }
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
