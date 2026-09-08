use crate::{Buffer, Config, Input, Processor, Result, SimpleRenderer};

const SUFFIX_SPACE: &str = " ";

/// Deterministic, side-effect-free Vietnamese input engine.
pub struct Engine<P: Processor> {
    config: Config,
    processor: P,
    buffer: Buffer,
}

impl Default for Engine<SimpleRenderer> {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl<P: Processor> Engine<P> {
    pub fn with_processor(config: Config, processor: P) -> Self {
        Self {
            config,
            processor,
            buffer: Buffer::new(),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn rendered(&self) -> String {
        self.processor
            .render(self.buffer.raw_chars(), self.buffer.cursor())
    }

    pub fn reset(&mut self) -> Result {
        self.buffer.clear();
        Result::Changed
    }

    pub fn input(&mut self, input: Input) -> Result {
        match input {
            Input::Character(character) => self.insert(character),
            Input::Backspace => self.erase(Buffer::backspace),
            Input::Delete => self.erase(Buffer::delete),
            Input::Left => self.move_cursor(Buffer::move_left),
            Input::Right => self.move_cursor(Buffer::move_right),
            Input::Space => self.commit_with_suffix(SUFFIX_SPACE),
            Input::Enter | Input::Tab | Input::Escape => self.commit(),
        }
    }

    pub fn commit(&mut self) -> Result {
        if self.buffer.is_empty() {
            return Result::Forward;
        }
        let rendered = self.rendered();
        self.buffer.clear();
        Result::Commit(rendered)
    }

    fn commit_with_suffix(&mut self, suffix: &str) -> Result {
        if self.buffer.is_empty() {
            return Result::Forward;
        }
        let mut text = self.rendered();
        text.push_str(suffix);
        self.buffer.clear();
        Result::Commit(text)
    }

    fn insert(&mut self, character: char) -> Result {
        self.buffer.insert(character);
        Result::Changed
    }

    fn erase(&mut self, operation: fn(&mut Buffer)) -> Result {
        if self.buffer.is_empty() {
            return Result::Forward;
        }
        operation(&mut self.buffer);
        Result::Changed
    }

    fn move_cursor(&mut self, operation: fn(&mut Buffer)) -> Result {
        operation(&mut self.buffer);
        Result::Changed
    }
}

impl Engine<SimpleRenderer> {
    pub fn new(config: Config) -> Self {
        Self::with_processor(config, SimpleRenderer::default())
    }
}
