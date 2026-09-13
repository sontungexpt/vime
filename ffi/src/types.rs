use std::ffi::c_char;
use std::ptr;

use vime_engine::{ConfiguredRuleEngine, DefaultRenderer, Engine};

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VimeInputMethod {
    #[default]
    Telex = 1,
    Vni = 2,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VimeKey {
    #[default]
    None = 0,
    Backspace = 1,
    Delete = 2,
    Left = 3,
    Right = 4,
    Enter = 5,
    Escape = 6,
    Tab = 7,
    Space = 8,
}

#[repr(C)]
pub struct VimeEngineHandle {
    pub engine: Engine<DefaultRenderer, ConfiguredRuleEngine<'static>>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VimeOutput {
    pub consumed: bool,
    pub changed: bool,
    pub rendered: *mut c_char,
    pub commit: *mut c_char,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VimeKeyEvent {
    pub key: VimeKey,
    pub character: u32,
    pub states: u32,
}

impl VimeOutput {
    #[inline]
    pub const fn empty() -> Self {
        Self {
            consumed: false,
            changed: false,
            rendered: ptr::null_mut(),
            commit: ptr::null_mut(),
        }
    }
}

impl Default for VimeOutput {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}
