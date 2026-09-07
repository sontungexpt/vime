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
mod processor;
mod renderer;
mod result;
mod state;

pub mod character;

pub use character::{decode_vowel, encode_vowel, BaseVowel, Case, RootVowel, Shape, Tone, Vowel};
pub use composition::Composition;
pub use config::Config;
pub use engine::Engine;
pub use input::Input;
pub use interpreter::{
    SimpleInterpreter, Interpreter, InterpreterConfig, KeyContext, Operation, ShapeConfig,
    ShapeTarget, ToneConfig,
};
pub use processor::{
    analyze_syllable, analyze_syllable_with_orthography, canonicalize_into, normalize, Coda, Onset,
    Orthography, Processor, SequenceState, SyllableAnalysis, WordStructure,
};
pub use renderer::{Renderer, SimpleRenderer};
pub use result::Result;
pub use state::State;
