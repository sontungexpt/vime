//! Frontend-independent Vietnamese input method engine.
//!
//! Architecture:
//!
//! ```text
//! Input → Interpreter → Action → Processor → Unicode
//! ```
//!
//! - [`character`]: semantic Vietnamese vowels as primitive `(base, tone, case)`
//!   triples and the [`decode`]/[`encode`] codec over precomposed characters.
//! - [`Interpreter`]: turns a key plus [`interpreter::KeyContext`] into an
//!   [`Action`], per input-method configuration (`SimpleInterpreter`).
//! - [`Action`]: the semantic model of a keystroke.
//! - [`Processor`]: Vietnamese rules; applies actions to semantic vowels,
//!   and parses/renders canonical ASCII syllables into Vietnamese text.
//! - [`Buffer`] + [`Engine`]: raw input buffering, cursor editing, and the
//!   frontend-facing state machine.

mod buffer;
mod config;
mod engine;
mod input;
mod interpreter;
pub mod processor;
mod renderer;
mod result;

pub mod phonology;

pub use buffer::Buffer;
pub use config::Config;
pub use engine::Engine;
pub use input::Input;
pub use interpreter::{
    Action, Interpreter, InterpreterConfig, KeyContext, ShapeConfig, ShapeFamily,
    SimpleInterpreter, ToneConfig,
};
pub use phonology::{
    decode_vowel, encode_vowel, is_vowel, BaseVowel, Case, RootVowel, Shape, Tone,
};
pub use processor::Processor;
pub use renderer::{Orthography, ParsedSyllable, Renderer, SimpleRenderer};
pub use result::Result;
