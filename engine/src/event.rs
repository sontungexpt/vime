use bitflags::bitflags;

bitflags! {
    /// Bitmask matching fcitx5's `KeyState` layout exactly so the C++ adapter
    /// can pass through `static_cast<uint32_t>(key.states())` without translation.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct KeyState: u32 {
        const SHIFT     = 1 << 0;
        const CAPS_LOCK = 1 << 1;
        const CTRL      = 1 << 2;
        const ALT       = 1 << 3;
        const NUM_LOCK  = 1 << 4;
        const HYPER     = 1 << 5;
        const SUPER     = 1 << 6;
        const META      = 1 << 28;
    }
}

/// A physical key press, independent of modifier state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    Character(char),
    Backspace,
    Delete,
    Left,
    Right,
    Enter,
    Escape,
    Tab,
    Space,
}

/// A fully described keyboard event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyEvent {
    pub key: Key,
    pub state: KeyState,
}

impl KeyEvent {
    /// Creates an event with no modifier keys held.
    #[inline]
    pub const fn key(key: Key) -> Self {
        Self {
            key,
            state: KeyState::empty(),
        }
    }
}
