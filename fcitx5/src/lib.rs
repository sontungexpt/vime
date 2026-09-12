use std::ffi::{c_char, CString};
use std::ptr;
use vime_engine::{
    DefaultKeyMapping, DefaultRenderer, Engine, Key, KeyEvent, KeyState, Result,
};

#[repr(C)]
pub struct VimeEngineHandle {
    engine: Engine<DefaultRenderer, DefaultKeyMapping<'static>>,
}

#[repr(C)]
pub struct VimeOutput {
    pub consumed: bool,
    pub changed: bool,
    pub rendered: *mut c_char,
    pub commit: *mut c_char,
}

#[repr(C)]
pub struct VimeKeyEvent {
    pub key: u32,
    pub character: u32,
    pub state: u32,
}

impl VimeOutput {
    fn empty() -> Self {
        Self {
            consumed: false,
            changed: false,
            rendered: ptr::null_mut(),
            commit: ptr::null_mut(),
        }
    }
}

#[no_mangle]
pub extern "C" fn vime_create() -> *mut VimeEngineHandle {
    Box::into_raw(Box::new(VimeEngineHandle {
        engine: Engine::default(),
    }))
}

#[no_mangle]
/// Destroys an engine previously returned by `vime_create`.
///
/// # Safety
///
/// `engine` must be null or a valid, not-yet-freed engine pointer.
pub unsafe extern "C" fn vime_destroy(engine: *mut VimeEngineHandle) {
    if !engine.is_null() {
        drop(Box::from_raw(engine));
    }
}

#[no_mangle]
/// Resets the semantic engine state.
///
/// # Safety
///
/// `engine` must be null or a valid engine pointer.
pub unsafe extern "C" fn vime_reset(engine: *mut VimeEngineHandle) -> VimeOutput {
    let Some(engine) = engine.as_mut() else {
        return VimeOutput::empty();
    };
    let result = engine.engine.reset();
    output(&engine.engine, result)
}

#[no_mangle]
/// Processes a single keyboard event (key + modifiers + character).
///
/// # Safety
///
/// `engine` must be null or a valid engine pointer.
pub unsafe extern "C" fn vime_process_key(
    engine: *mut VimeEngineHandle,
    event: VimeKeyEvent,
) -> VimeOutput {
    let Some(engine) = engine.as_mut() else {
        return VimeOutput::empty();
    };

    let key = if event.key != 0 {
        match event.key {
            1 => Key::Backspace,
            2 => Key::Delete,
            3 => Key::Left,
            4 => Key::Right,
            5 => Key::Enter,
            6 => Key::Escape,
            7 => Key::Tab,
            8 => Key::Space,
            _ => return VimeOutput::empty(),
        }
    } else if let Some(ch) = char::from_u32(event.character) {
        Key::Character(ch)
    } else {
        return VimeOutput::empty();
    };

    let result = engine.engine.process_key(KeyEvent {
        key,
        state: KeyState::from_bits_truncate(event.state),
    });
    output(&engine.engine, result)
}

#[no_mangle]
/// Changes the input method used by the semantic engine.
///
/// # Safety
///
/// `engine` must be null or a valid engine pointer.
pub unsafe extern "C" fn vime_set_method(engine: *mut VimeEngineHandle, method: u32) {
    if let Some(engine) = engine.as_mut() {
        engine.engine.set_layout(match method {
            1 => DefaultKeyMapping::telex(), // VIME_INPUT_METHOD_TELEX
            2 => DefaultKeyMapping::vni(),   // VIME_INPUT_METHOD_VNI
            _ => return,
        });
    }
}

#[no_mangle]
/// Frees a string returned in `VimeOutput`.
///
/// # Safety
///
/// `value` must be null or allocated by this crate and must not be freed twice.
pub unsafe extern "C" fn vime_free_string(value: *mut c_char) {
    if !value.is_null() {
        drop(CString::from_raw(value));
    }
}

fn output(
    engine: &Engine<DefaultRenderer, DefaultKeyMapping<'static>>,
    result: Result,
) -> VimeOutput {
    let mut output = VimeOutput::empty();
    match result {
        Result::Changed => {
            output.consumed = true;
            output.changed = true;
            output.rendered = CString::new(engine.rendered())
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
