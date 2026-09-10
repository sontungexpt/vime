use crate::{
    renderer::parser::ParseStatus, Buffer, BufferChar, Config, DefaultKeyMapping, DefaultRenderer,
    Input, KeyMapping, Parser, Renderer, Result,
};

const SUFFIX_SPACE: &str = " ";

mod api;

pub use api::InputEngine;

pub struct Engine<R: Renderer, I: KeyMapping> {
    config: Config,
    keystrokes: Buffer,
    mapping: I,
    renderer: R,
}

impl<R, I> Engine<R, I>
where
    R: Renderer,
    I: KeyMapping + Clone,
{
    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn keystrokes(&self) -> &Buffer {
        &self.keystrokes
    }

    pub fn rendered(&self) -> String {
        let mut parser = Parser::new(&self.mapping);
        parser.parse(self.keystrokes.chars());

        if let ParseStatus::Dead(_) = parser.status() {
            return self.keystrokes.to_string();
        }

        self.renderer.render(parser.syllable())
    }

    pub fn reset(&mut self) -> Result {
        self.keystrokes.clear();
        Result::Changed
    }

    pub fn input(&mut self, input: Input) -> Result {
        match input {
            Input::Character(character) => self.insert(character),
            Input::Backspace => self.backspace(),
            Input::Delete => self.delete(),
            Input::Left => self.move_left(),
            Input::Right => self.move_right(),
            Input::Space => self.commit_with_suffix(SUFFIX_SPACE),
            Input::Enter | Input::Tab | Input::Escape => self.commit(),
        }
    }

    pub fn commit(&mut self) -> Result {
        self.commit_with_suffix("")
    }

    fn commit_with_suffix(&mut self, suffix: &str) -> Result {
        if self.keystrokes.is_empty() {
            return Result::Forward;
        }

        let mut text = self.rendered();
        text.push_str(suffix);
        self.keystrokes.clear();
        Result::Commit(text)
    }

    fn insert(&mut self, character: char) -> Result {
        if self.mapping.is_transform(character) {
            self.keystrokes.insert(BufferChar::Transform(character));
        } else {
            self.keystrokes.insert(BufferChar::Literal(character));
        }
        Result::Changed
    }

    fn backspace(&mut self) -> Result {
        self.apply(Buffer::backspace)
    }

    fn delete(&mut self) -> Result {
        self.apply(Buffer::delete)
    }

    fn move_left(&mut self) -> Result {
        self.apply(Buffer::move_left)
    }

    fn move_right(&mut self) -> Result {
        self.apply(Buffer::move_right)
    }

    fn apply(&mut self, operation: fn(&mut Buffer)) -> Result {
        if self.keystrokes.is_empty() {
            return Result::Forward;
        }

        operation(&mut self.keystrokes);
        Result::Changed
    }
}

impl<R, I> InputEngine for Engine<R, I>
where
    R: Renderer,
    I: KeyMapping + Clone,
{
    fn input(&mut self, input: Input) -> Result {
        self.input(input)
    }

    fn commit(&mut self) -> Result {
        self.commit()
    }

    fn reset(&mut self) -> Result {
        self.reset()
    }

    fn rendered(&self) -> String {
        self.rendered()
    }

    fn keystrokes(&self) -> &Buffer {
        self.keystrokes()
    }

    fn config(&self) -> &Config {
        self.config()
    }
}

impl Engine<DefaultRenderer, DefaultKeyMapping<'static>> {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            keystrokes: Buffer::new(),
            mapping: DefaultKeyMapping::telex(),
            renderer: DefaultRenderer::default(),
        }
    }
}

impl Default for Engine<DefaultRenderer, DefaultKeyMapping<'static>> {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
