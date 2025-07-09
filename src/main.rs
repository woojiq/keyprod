use keyprod::kbd_events::{KbdEventListener, LinuxKeyboardEventListener, get_all_kbd_devices};

fn main() {
    let (tx, rx) = std::sync::mpsc::channel::<keyprod::kbd_events::KbdEvent>();

    let devices = get_all_kbd_devices().unwrap();

    let mut listener = LinuxKeyboardEventListener::new(tx, devices);
    listener.listen();
}
