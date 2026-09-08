pub trait Processor {
    fn render(&self, raw: &[char], cursor: usize) -> String;
}
