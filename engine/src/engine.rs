use crate::{
    parser::ParseStatus, Buffer, Config, DefaultKeyMapping, DefaultRenderer, Input,
    KeyMapping, Parser, Renderer, Result,
};

const SUFFIX_SPACE: &str = " ";

mod api;

pub use api::InputEngine;

/// The core input method state machine: buffers raw keystrokes and renders
/// them into Vietnamese text.
pub struct Engine<R: Renderer> {
    config: Config,
    keystrokes: Buffer,
    renderer: R,
    layout: DefaultKeyMapping<'static>,
}

impl<R> Engine<R>
where
    R: Renderer,
{
    /// The engine configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// The raw keystroke buffer.
    pub fn keystrokes(&self) -> &Buffer {
        &self.keystrokes
    }

    /// Renders the current buffer as Vietnamese text, or as the raw ASCII
    /// keystrokes when the buffer cannot form a valid syllable.
    pub fn rendered(&self) -> String {
        let mut parser = Parser::new(self.layout);
        let (status, _) = parser.parse(self.keystrokes.chars());

        if let ParseStatus::Dead(_) = status {
            return self.keystrokes.to_string();
        }

        self.renderer.render(parser.syllable())
    }

    /// The input layout the engine is currently using.
    #[inline(always)]
    pub const fn layout(&self) -> DefaultKeyMapping<'static> {
        self.layout
    }

    /// Switches the engine to `layout` for subsequent renders and commits.
    #[inline(always)]
    pub fn set_layout(&mut self, layout: DefaultKeyMapping<'static>) {
        self.layout = layout;
    }

    /// Clears the buffer.
    pub fn reset(&mut self) -> Result {
        self.keystrokes.clear();
        Result::Changed
    }

    /// Feeds one input into the engine.
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

    /// Commits the current buffer and returns the resulting text.
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
        self.keystrokes.insert(character);
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

impl<R> InputEngine for Engine<R>
where
    R: Renderer,
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

impl Engine<DefaultRenderer> {
    /// Creates a Telex engine with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            config,
            keystrokes: Buffer::new(),
            renderer: DefaultRenderer::default(),
            layout: DefaultKeyMapping::telex(),
        }
    }

    /// Creates an engine using `layout` with the default configuration.
    #[inline(always)]
    pub fn with_layout(layout: DefaultKeyMapping<'static>) -> Self {
        Self {
            config: Config::default(),
            keystrokes: Buffer::new(),
            renderer: DefaultRenderer::default(),
            layout,
        }
    }
}

impl Default for Engine<DefaultRenderer> {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
