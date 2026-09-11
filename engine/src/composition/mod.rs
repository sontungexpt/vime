//! Raw input buffering and syllable composition.
//!
//! - [`buffer::Buffer`]: the editable list of raw keystrokes as typed.
//! - [`syllable::Syllable`]: a Vietnamese syllable under construction.

pub mod buffer;
pub mod syllable;

pub use buffer::Buffer;
pub use syllable::{Cased, Syllable};