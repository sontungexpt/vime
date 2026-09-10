use crate::Syllable;

/// Renders canonical raw Vietnamese input into Unicode Vietnamese text.
pub trait Renderer {
    fn render(&self, syllable: &Syllable) -> String;
}
