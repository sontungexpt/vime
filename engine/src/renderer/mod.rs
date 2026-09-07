use crate::processor::render_raw;
use crate::{Interpreter, Orthography, SimpleInterpreter};

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
    fn render(&self, raw: &[char], _cursor: usize) -> String {
        if raw.len() <= 1 {
            return raw.iter().collect();
        }
        let mut input = String::with_capacity(raw.len());
        input.extend(raw);
        render_raw(&input, self.orthography)
    }
}

impl Default for SimpleRenderer<SimpleInterpreter<'static>> {
    fn default() -> Self {
        Self::new(SimpleInterpreter::telex(), Orthography::default())
    }
}