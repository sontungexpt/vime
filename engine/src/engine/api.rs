use crate::{Buffer, Config, Input, Result};

/// Frontend-facing interface of an input method engine.
pub trait InputEngine {
    /// Feed one input into the engine.
    fn input(&mut self, input: Input) -> Result;

    /// Finalize the current buffer and return the committed text.
    fn commit(&mut self) -> Result;

    /// Clear the current buffer.
    fn reset(&mut self) -> Result;

    /// Rendered Vietnamese text of the current buffer.
    fn rendered(&self) -> String;

    /// Raw keystroke buffer.
    fn keystrokes(&self) -> &Buffer;

    /// Engine configuration.
    fn config(&self) -> &Config;
}