use keyprod::{
    kbd_event_listener::{KbdEventListener, LinuxKeyboardEventListener, get_all_kbd_devices},
    publisher::{DefaultKeyboardEventPublisher, KeyboardEventPublisher},
};

fn main() {
    let (tx, rx) = std::sync::mpsc::channel::<keyprod::kbd_event::KbdEvent>();

    let devices = get_all_kbd_devices().unwrap();

    let listener_handler = std::thread::spawn(move || {
        let mut listener = LinuxKeyboardEventListener::new(tx, devices);
        listener.listen();
    });

    let publisher_handler = std::thread::spawn(move || {
        let mut publisher = DefaultKeyboardEventPublisher::new(rx);

        let event_logger = Box::new(keyprod::subscribers::EventLogger::new_stdout());
        publisher.register_subscriber(event_logger);

        publisher.run();
    });

    listener_handler.join().unwrap();
    publisher_handler.join().unwrap();
}
