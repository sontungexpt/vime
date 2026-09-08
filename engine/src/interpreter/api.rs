use super::Operation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyContext {
    pub target: Option<char>,
}

/// Interprets keyboard input into semantic Vietnamese operations.
pub trait Interpreter {
    fn interpret(&self, context: KeyContext, input: char) -> Operation;
}
