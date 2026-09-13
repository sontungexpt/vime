use std::ffi::CString;

use vime_engine::{ConfiguredRuleEngine, DefaultRenderer, Engine, Key, KeyEvent, KeyState, Result};

use crate::types::{VimeKey, VimeKeyEvent, VimeOutput};

impl TryFrom<VimeKeyEvent> for KeyEvent {
    type Error = ();

    #[inline]
    fn try_from(event: VimeKeyEvent) -> std::result::Result<Self, Self::Error> {
        let key = match event.key {
            VimeKey::None => char::from_u32(event.character)
                .map(Key::Character)
                .ok_or(())?,
            VimeKey::Backspace => Key::Backspace,
            VimeKey::Delete => Key::Delete,
            VimeKey::Left => Key::Left,
            VimeKey::Right => Key::Right,
            VimeKey::Enter => Key::Enter,
            VimeKey::Escape => Key::Escape,
            VimeKey::Tab => Key::Tab,
            VimeKey::Space => Key::Space,
        };

        Ok(KeyEvent {
            key,
            states: KeyState::from_bits_truncate(event.states),
        })
    }
}

pub fn to_vime_output(
    engine: &Engine<DefaultRenderer, ConfiguredRuleEngine<'static>>,
    result: Result,
) -> VimeOutput {
    let mut output = VimeOutput::empty();
    match result {
        Result::Changed => {
            output.consumed = true;
            output.changed = true;
            let rendered = engine.rendered();
            output.rendered = CString::new(rendered)
                .expect("rendered text cannot contain NUL")
                .into_raw();
        }
        Result::Commit(text) => {
            output.consumed = true;
            output.changed = true;
            output.rendered = CString::new("")
                .expect("empty text cannot contain NUL")
                .into_raw();
            output.commit = CString::new(text)
                .expect("committed text cannot contain NUL")
                .into_raw();
        }
        Result::Noop => {}
        Result::Forward => {}
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vime_key_event_to_rust_key_event() {
        let event = VimeKeyEvent {
            key: VimeKey::Backspace,
            character: 0,
            states: 0,
        };
        let rust_event = KeyEvent::try_from(event).unwrap();
        assert_eq!(rust_event.key, Key::Backspace);
        assert_eq!(rust_event.states, KeyState::empty());

        let event = VimeKeyEvent {
            key: VimeKey::None,
            character: 'v' as u32,
            states: 0,
        };
        let rust_event = KeyEvent::try_from(event).unwrap();
        assert_eq!(rust_event.key, Key::Character('v'));
        assert_eq!(rust_event.states, KeyState::empty());

        let invalid_event = VimeKeyEvent {
            key: VimeKey::None,
            character: 0x110000,
            states: 0,
        };
        assert!(KeyEvent::try_from(invalid_event).is_err());
    }
}
