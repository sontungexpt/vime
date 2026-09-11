use std::ffi::{c_char, CString};
use std::ptr;
use vime_engine::{DefaultRenderer, DefaultKeyMapping, Engine, Input, Result};

#[repr(C)]
pub struct VietnameseFcitx5Engine {
    engine: Engine<DefaultRenderer>,
}

#[repr(C)]
pub struct VietnameseFcitx5Output {
    pub consumed: bool,
    pub changed: bool,
    pub rendered: *mut c_char,
    pub commit: *mut c_char,
}

impl VietnameseFcitx5Output {
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
pub extern "C" fn vietnamese_fcitx5_create() -> *mut VietnameseFcitx5Engine {
    Box::into_raw(Box::new(VietnameseFcitx5Engine {
        engine: Engine::default(),
    }))
}

#[no_mangle]
/// Destroys an engine previously returned by `vietnamese_fcitx5_create`.
///
/// # Safety
///
/// `engine` must be null or a valid, not-yet-freed engine pointer.
pub unsafe extern "C" fn vietnamese_fcitx5_destroy(engine: *mut VietnameseFcitx5Engine) {
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
pub unsafe extern "C" fn vietnamese_fcitx5_reset(
    engine: *mut VietnameseFcitx5Engine,
) -> VietnameseFcitx5Output {
    let Some(engine) = engine.as_mut() else {
        return VietnameseFcitx5Output::empty();
    };
    let result = engine.engine.reset();
    output(&engine.engine, result)
}

#[no_mangle]
/// Processes one Unicode scalar value.
///
/// # Safety
///
/// `engine` must be null or a valid engine pointer.
pub unsafe extern "C" fn vietnamese_fcitx5_process_character(
    engine: *mut VietnameseFcitx5Engine,
    character: u32,
) -> VietnameseFcitx5Output {
    let Some(engine) = engine.as_mut() else {
        return VietnameseFcitx5Output::empty();
    };
    let Some(character) = char::from_u32(character) else {
        return VietnameseFcitx5Output::empty();
    };
    let result = engine.engine.input(Input::Character(character));
    output(&engine.engine, result)
}

#[no_mangle]
/// Processes an editing or boundary input.
///
/// # Safety
///
/// `engine` must be null or a valid engine pointer.
pub unsafe extern "C" fn vietnamese_fcitx5_process_key(
    engine: *mut VietnameseFcitx5Engine,
    key: u32,
) -> VietnameseFcitx5Output {
    let Some(engine) = engine.as_mut() else {
        return VietnameseFcitx5Output::empty();
    };
    let input = match key {
        1 => Input::Backspace,
        2 => Input::Delete,
        3 => Input::Left,
        4 => Input::Right,
        5 => Input::Enter,
        6 => Input::Escape,
        7 => Input::Tab,
        8 => Input::Space,
        _ => return VietnameseFcitx5Output::empty(),
    };
    let result = engine.engine.input(input);
    output(&engine.engine, result)
}

#[no_mangle]
/// Changes the input method used by the semantic engine.
///
/// # Safety
///
/// `engine` must be null or a valid engine pointer.
pub unsafe extern "C" fn vietnamese_fcitx5_set_method(
    engine: *mut VietnameseFcitx5Engine,
    vni: bool,
) {
    if let Some(engine) = engine.as_mut() {
        engine.engine.set_layout(if vni {
            DefaultKeyMapping::vni()
        } else {
            DefaultKeyMapping::telex()
        });
    }
}

#[no_mangle]
/// Frees a string returned in `VietnameseFcitx5Output`.
///
/// # Safety
///
/// `value` must be null or allocated by this crate and must not be freed twice.
pub unsafe extern "C" fn vietnamese_fcitx5_free_string(value: *mut c_char) {
    if !value.is_null() {
        drop(CString::from_raw(value));
    }
}

fn output(engine: &Engine<DefaultRenderer>, result: Result) -> VietnameseFcitx5Output {
    let mut output = VietnameseFcitx5Output::empty();
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
