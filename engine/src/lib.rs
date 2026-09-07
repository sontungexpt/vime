//! Frontend-independent Vietnamese input method engine.
//!
//! Architecture:
//!
//! ```text
//! Input → Interpreter → Operation → Processor → Unicode
//! ```
//!
//! - [`character`]: semantic Vietnamese vowels as primitive `(base, tone, case)`
//!   triples and the [`decode`]/[`encode`] codec over precomposed characters.
//! - [`Interpreter`]: turns a key plus [`interpreter::KeyContext`] into an
//!   [`Operation`], per input-method configuration (`SimpleInterpreter`).
//! - [`Operation`]: the semantic model of a keystroke.
//! - [`Processor`]: Vietnamese rules; applies operations to semantic vowels,
//!   and parses/renders canonical ASCII syllables into Vietnamese text.
//! - [`Composition`] + [`Engine`]: raw composition, cursor editing, and the
//!   frontend-facing state machine.

mod composition;
mod config;
mod engine;
mod input;
mod interpreter;
mod renderer;
mod result;

pub mod phonology;

pub use composition::Composition;
pub use config::Config;
pub use engine::Engine;
pub use input::Input;
pub use interpreter::{
    Interpreter, InterpreterConfig, KeyContext, Operation, ShapeConfig, ShapeTarget,
    SimpleInterpreter, ToneConfig,
};
pub use phonology::{
    decode_vowel, encode_vowel, is_vowel, BaseVowel, Case, RootVowel, Shape, Tone,
};
pub use renderer::{Orthography, ParsedSyllable, Renderer, SimpleRenderer};
pub use result::Result;
