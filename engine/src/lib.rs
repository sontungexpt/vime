//! Frontend-independent Vietnamese input method engine.
//!
//! Architecture:
//!
//! ```text
//! Input → Interpreter → Processor → Unicode
//! ```
//!
//! - [`character`]: semantic Vietnamese vowels as primitive `(base, tone, case)`
//!   triples and the [`decode`]/[`encode`] codec over precomposed characters.
//! - [`Interpreter`]: turns a key plus [`interpreter::KeyContext`] into
//!   shape or tone changes, per input-method configuration
//!   (`SimpleInterpreter`).
//! - [`Processor`]: Vietnamese rules; applies actions to semantic vowels,
//!   and parses/renders canonical ASCII syllables into Vietnamese text.
//! - [`Buffer`] + [`Engine`]: raw input buffering, cursor editing, and the
//!   frontend-facing state machine.

mod buffer;
mod config;
mod engine;
mod input;
mod interpreter;
pub mod renderer;
mod result;

pub mod phonology;

pub use buffer::{Buffer, BufferChar};
pub use config::Config;
pub use engine::{Engine, InputEngine};
pub use input::Input;
pub use interpreter::{
    DefaultKeyMapping, KeyConfig, KeyMapping, KeyTarget, ShapeMap, ToneMap,
};
pub use phonology::{
    decode_vowel, encode_vowel, is_vowel, BaseVowel, Case, RootVowel, Shape, Tone,
};
pub use renderer::{
    Cased, Orthography, Parser, Renderer, SimpleRenderer, Syllable,
};
pub use result::Result;
