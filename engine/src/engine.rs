use crate::{
    Buffer, BufferChar, Config, Input, KeyInterpreter, Renderer, Result, SimpleInterpreter,
    SimpleRenderer,
};

const SUFFIX_SPACE: &str = " ";

pub struct Engine<R: Renderer, I: KeyInterpreter> {
    config: Config,
    keystrokes: Buffer,
    interpreter: I,
    renderer: R,
}

impl<R, I> Engine<R, I>
where
    R: Renderer,
    I: KeyInterpreter,
{
    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn keystrokes(&self) -> &Buffer {
        &self.keystrokes
    }

    pub fn rendered(&self) -> String {
        let raw: Vec<char> = self
            .keystrokes
            .chars()
            .iter()
            .map(BufferChar::as_char)
            .collect();
        self.renderer.render(&raw, self.keystrokes.cursor())
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
        if self.interpreter.is_transform_key(character) {
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

impl Engine<SimpleRenderer, SimpleInterpreter<'static>> {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            keystrokes: Buffer::new(),
            interpreter: SimpleInterpreter::telex(),
            renderer: SimpleRenderer::default(),
        }
    }
}

impl Default for Engine<SimpleRenderer, SimpleInterpreter<'static>> {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
