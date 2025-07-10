// https://github.com/torvalds/linux/blob/master/include/uapi/linux/input.h
// Short explanation of fields: https://stackoverflow.com/a/16695758/17903686
// All types & codes: https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/include/uapi/linux/input-event-codes.h
// value: 0 (release), 1 (keypress), 2 (repeat).

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct InputEvent(libc::input_event);

#[derive(Debug, Copy, Clone)]
pub struct KbdEvent {
    pub code: u16,
    pub state: KbdKeyState,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KbdKeyState {
    Released = 0,
    Pressed = 1,
    Repeated = 2,
}

impl InputEvent {
    const EV_KEY: u16 = 0x01;
    /// # Safety
    ///
    /// Bytes must form correct keyboard events (as device files guarantee).
    pub unsafe fn from_raw_bytes(buff: &[u8]) -> Vec<InputEvent> {
        assert_eq!(buff.len() % size_of::<Self>(), 0);
        let events_cnt = buff.len() / size_of::<Self>();

        let mut events = Vec::with_capacity(events_cnt);
        unsafe {
            let ptr = buff.as_ptr() as *const Self;
            let slice = std::slice::from_raw_parts(ptr, events_cnt);
            events.extend_from_slice(slice);
        }

        events
    }
}

impl TryFrom<InputEvent> for KbdEvent {
    type Error = ();

    fn try_from(value: InputEvent) -> Result<Self, Self::Error> {
        if value.0.type_ == InputEvent::EV_KEY {
            Ok(KbdEvent {
                code: value.0.code,
                state: KbdKeyState::try_from(value.0.value)?,
            })
        } else {
            Err(())
        }
    }
}

impl KbdEvent {
    pub fn is_release(&self) -> bool {
        self.state == KbdKeyState::Released
    }
    pub fn is_press(&self) -> bool {
        self.state == KbdKeyState::Pressed
    }
    pub fn is_repeat(&self) -> bool {
        self.state == KbdKeyState::Repeated
    }
}

impl TryFrom<i32> for KbdKeyState {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            x if x == Self::Released as i32 => Ok(Self::Released),
            x if x == Self::Pressed as i32 => Ok(Self::Pressed),
            x if x == Self::Repeated as i32 => Ok(Self::Repeated),
            _ => Err(()),
        }
    }
}
