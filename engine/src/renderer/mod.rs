mod api;
pub mod parser;

pub use api::Renderer;
pub use parser::ParsedSyllable;

use crate::{KeyInterpreter, SimpleInterpreter};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Orthography {
    /// Today's standard tone placement (e.g. *hòa*, *tuyển*).
    #[default]
    Modern,
    /// Traditional (pre-reform) tone placement (e.g. *hoà*, *thuý*).
    Old,
}

/// The standard renderer, backed by an interpreter and an orthography.
pub struct SimpleRenderer<I: KeyInterpreter = SimpleInterpreter<'static>> {
    interpreter: I,
    orthography: Orthography,
}

impl<I: KeyInterpreter> SimpleRenderer<I> {
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

impl Default for SimpleRenderer<SimpleInterpreter<'static>> {
    fn default() -> Self {
        Self::new(SimpleInterpreter::telex(), Orthography::default())
    }
}

impl<I: KeyInterpreter> Renderer for SimpleRenderer<I> {
    fn render(&self, raw: &[char], _cursor: usize) -> String {
        raw.iter().collect()
    }
}
