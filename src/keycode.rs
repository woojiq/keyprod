pub mod raw {
    include!(concat!(env!("OUT_DIR"), "/input_bindings_raw.rs"));
}

include!(concat!(env!("OUT_DIR"), "/keycode.rs"));

impl Keycode {
    const MODIFIERS: [Keycode; 8] = [
        Keycode::KEY_LEFTSHIFT,
        Keycode::KEY_RIGHTSHIFT,
        Keycode::KEY_LEFTCTRL,
        Keycode::KEY_RIGHTCTRL,
        Keycode::KEY_LEFTALT,
        Keycode::KEY_RIGHTALT,
        Keycode::KEY_LEFTMETA,
        Keycode::KEY_RIGHTMETA,
    ];

    pub fn is_modifier(&self) -> bool {
        Self::MODIFIERS.contains(self)
    }
}
