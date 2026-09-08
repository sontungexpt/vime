pub mod syllable;

pub use syllable::ParsedSyllable;

use crate::{processor, Interpreter, SimpleInterpreter};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Orthography {
    /// Today's standard tone placement (e.g. *hòa*, *tuyển*).
    #[default]
    Modern,
    /// Traditional (pre-reform) tone placement (e.g. *hoà*, *thuý*).
    Old,
}

/// Renders canonical raw Vietnamese input into Unicode Vietnamese text.
pub trait Renderer {
    fn render(&self, raw: &[char], cursor: usize) -> String;
}

/// The standard renderer, backed by an interpreter and an orthography.
pub struct SimpleRenderer<I: Interpreter = SimpleInterpreter<'static>> {
    interpreter: I,
    orthography: Orthography,
}

impl<I: Interpreter> SimpleRenderer<I> {
    pub const fn new(interpreter: I, orthography: Orthography) -> Self {
        Self {
            interpreter,
            orthography,
        }
    }

    pub const fn interpreter(&self) -> &I {
        &self.interpreter
    }

    pub const fn orthography(&self) -> Orthography {
        self.orthography
    }
}

impl<I: Interpreter> Renderer for SimpleRenderer<I> {
    fn render(&self, raw: &[char], cursor: usize) -> String {
        let _ = cursor;
        processor::render_raw(raw)
    }
}

impl Default for SimpleRenderer<SimpleInterpreter<'static>> {
    fn default() -> Self {
        Self::new(SimpleInterpreter::telex(), Orthography::default())
    }
}
