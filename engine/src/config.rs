use crate::{Interpreter, SimpleInterpreter, SimpleRenderer};

/// User-configurable engine behavior.
pub struct Config<I: Interpreter = SimpleInterpreter<'static>> {
    pub renderer: SimpleRenderer<I>,
}

impl<I: Interpreter> Config<I> {
    pub const fn new(renderer: SimpleRenderer<I>) -> Self {
        Self { renderer }
    }
}

impl Default for Config<SimpleInterpreter<'static>> {
    fn default() -> Self {
        Self {
            renderer: SimpleRenderer::default(),
        }
    }
}