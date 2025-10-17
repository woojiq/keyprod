use keyprod::{
    kbd_event_listener::{KbdEventListener, LinuxKeyboardEventListener, get_all_kbd_devices},
    publisher::{DefaultKeyboardEventPublisher, KeyboardEventPublisher},
};

extern "C" fn sigint_handler(_: libc::c_int) {
    keyprod::kbd_event_listener::stop_listening();
}

fn get_sigint_handler() -> libc::sighandler_t {
    sigint_handler as extern "C" fn(libc::c_int) as *mut libc::c_void as libc::sighandler_t
}

fn main() {
    unsafe {
        libc::signal(libc::SIGINT, get_sigint_handler());
    }

    let (tx, rx) = std::sync::mpsc::channel::<keyprod::kbd_event::KbdEvent>();

    let devices = get_all_kbd_devices().unwrap();

    let listener_handler = std::thread::spawn(move || {
        let mut listener = LinuxKeyboardEventListener::new(tx, devices);
        listener.listen();
    });

    let publisher_handler = std::thread::spawn(move || {
        // TODO: cli args to enable/disable without recompilation.
        let mut publisher = DefaultKeyboardEventPublisher::new(rx);

        let event_echo = Box::new(keyprod::subscribers::EventEcho::new_stdout());
        publisher.register_subscriber(event_echo);

        let event_history = Box::new(keyprod::subscribers::EventHistory::new());
        publisher.register_subscriber(event_history);

        publisher.run();
    });

    listener_handler.join().unwrap();
    publisher_handler.join().unwrap();
}
