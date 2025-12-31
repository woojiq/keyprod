use keyprod::{
    kbd_event_listener::{KbdEventListener, LinuxKeyboardEventListener, get_all_kbd_devices},
    publisher::{DefaultKeyboardEventPublisher, KeyboardEventPublisher},
};

extern "C" fn signal_handler(_: libc::c_int) {
    keyprod::kbd_event_listener::stop_listening();
}

fn get_signal_handler() -> libc::sighandler_t {
    signal_handler as extern "C" fn(libc::c_int) as *mut libc::c_void as libc::sighandler_t
}

fn main() {
    let args = keyprod::args::parse_args().unwrap();

    unsafe {
        libc::signal(libc::SIGINT, get_signal_handler());
        libc::signal(libc::SIGTERM, get_signal_handler());
    }

    let (tx, rx) = std::sync::mpsc::channel::<keyprod::kbd_event::KbdEvent>();

    let devices = get_all_kbd_devices().unwrap();

    let listener_handler = std::thread::spawn(move || {
        let mut listener = LinuxKeyboardEventListener::new(tx, devices);
        listener.listen();
    });

    let publisher_handler = std::thread::spawn(move || {
        let mut publisher = DefaultKeyboardEventPublisher::new(rx);

        let subscribers =
            keyprod::subscribers::KeyboardEventSubscriberFactory::create_subscribers(&args);

        for subscriber in subscribers {
            publisher.register_subscriber(subscriber);
        }

        publisher.run();
    });

    listener_handler.join().unwrap();
    publisher_handler.join().unwrap();
}
