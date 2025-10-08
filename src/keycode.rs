include!(concat!(env!("OUT_DIR"), "/keycode.rs"));

pub mod raw {
    include!(concat!(env!("OUT_DIR"), "/input_bindings_raw.rs"));
}

// const MODIFIERS: [u32; 8] = [
//     KEY_LEFTSHIFT,
//     KEY_RIGHTSHIFT,
//     KEY_LEFTCTRL,
//     KEY_RIGHTCTRL,
//     KEY_LEFTALT,
//     KEY_RIGHTALT,
//     KEY_LEFTMETA,
//     KEY_RIGHTMETA,
// ];

// pub fn is_modifier(val: u32) -> bool {
//     MODIFIERS.contains(&val)
// }
