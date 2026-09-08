/// Renders canonical raw Vietnamese input into Unicode Vietnamese text.
pub trait Renderer {
    fn render(&self, raw: &[char], cursor: usize) -> String;
}
