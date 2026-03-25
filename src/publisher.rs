use crate::kbd_event::KbdEvent;

pub trait KeyboardEventPublisher {
    fn run(&mut self);
}

struct DefaultKeyboardEventPublisher {
    kbd_event_rcv: std::sync::mpsc::Receiver<KbdEvent>,
    plugin_event_txs: Vec<tokio::sync::mpsc::Sender<crate::Event>>,
}

impl DefaultKeyboardEventPublisher {
    pub fn new(
        kbd_event_rcv: std::sync::mpsc::Receiver<KbdEvent>,
        plugin_event_txs: Vec<tokio::sync::mpsc::Sender<crate::Event>>,
    ) -> Self {
        Self {
            kbd_event_rcv,
            plugin_event_txs,
        }
    }

    fn send_to_all(&mut self, event: crate::Event) {
        for tx in &mut self.plugin_event_txs {
            // TODO: error handling
            let _ = tx.blocking_send(event);
        }
    }
}

impl KeyboardEventPublisher for DefaultKeyboardEventPublisher {
    fn run(&mut self) {
        while let Ok(kbd_event) = self.kbd_event_rcv.recv() {
            self.send_to_all(kbd_event.into());
        }

        log::info!("Stopping all plugins.");

        self.send_to_all(crate::Event::PluginStop);
    }
}

pub fn spawn_publisher_thread(
    kbd_rx: std::sync::mpsc::Receiver<crate::kbd_event::KbdEvent>,
    txs_to_plugins: Vec<tokio::sync::mpsc::Sender<crate::Event>>,
) -> anyhow::Result<std::thread::JoinHandle<()>> {
    let publisher = std::thread::spawn(move || {
        let mut publisher = DefaultKeyboardEventPublisher::new(kbd_rx, txs_to_plugins);

        publisher.run();
    });

    Ok(publisher)
}
