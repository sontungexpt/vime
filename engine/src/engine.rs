use crate::{Composition, Config, Input, Interpreter, Renderer, Result, SimpleInterpreter, SimpleRenderer};

/// Suffix appended when committing on the space bar.
const SUFFIX_SPACE: &str = " ";

/// Deterministic, side-effect-free Vietnamese input engine.
pub struct Engine<I: Interpreter = SimpleInterpreter<'static>> {
    config: Config<I>,
    composition: Composition,
}

impl Default for Engine<SimpleInterpreter<'static>> {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl<I: Interpreter> Engine<I> {
    pub fn new(config: Config<I>) -> Self {
        Self {
            config,
            composition: Composition::new(),
        }
    }

    pub fn config(&self) -> &Config<I> {
        &self.config
    }

    pub fn set_renderer(&mut self, renderer: SimpleRenderer<I>) {
        self.config.renderer = renderer;
    }

    pub fn composition(&self) -> &Composition {
        &self.composition
    }

    /// Renders the canonical raw buffer into Vietnamese text.
    pub fn rendered(&self) -> String {
        self.config
            .renderer
            .render(self.composition.raw_chars(), self.composition.cursor())
    }

    pub fn reset(&mut self) -> Result {
        self.composition.clear();
        Result::Changed
    }

    pub fn input(&mut self, input: Input) -> Result {
        match input {
            Input::Character(character) => self.insert(character),
            Input::Backspace => self.erase(Composition::backspace),
            Input::Delete => self.erase(Composition::delete),
            Input::Left => self.move_cursor(Composition::move_left),
            Input::Right => self.move_cursor(Composition::move_right),
            Input::Home => self.move_cursor(Composition::move_home),
            Input::End => self.move_cursor(Composition::move_end),
            Input::Space => self.commit_with_suffix(SUFFIX_SPACE),
            Input::Enter | Input::Tab | Input::Escape => self.commit(),
        }
    }

    pub fn commit(&mut self) -> Result {
        if self.composition.is_empty() {
            return Result::Forward;
        }
        let rendered = self.rendered();
        self.composition.clear();
        Result::Commit(rendered)
    }

    fn commit_with_suffix(&mut self, suffix: &str) -> Result {
        if self.composition.is_empty() {
            return Result::Forward;
        }
        let mut text = self.rendered();
        text.push_str(suffix);
        self.composition.clear();
        Result::Commit(text)
    }

    #[inline]
    fn insert(&mut self, character: char) -> Result {
        self.composition.insert(character);
        Result::Changed
    }

    #[inline]
    fn erase(&mut self, operation: fn(&mut Composition)) -> Result {
        if self.composition.is_empty() {
            return Result::Forward;
        }
        operation(&mut self.composition);
        Result::Changed
    }

    #[inline]
    fn move_cursor(&mut self, operation: fn(&mut Composition)) -> Result {
        operation(&mut self.composition);
        Result::Changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Orthography;
    use crate::SimpleInterpreter;

    fn engine() -> Engine {
        Engine::default()
    }

    fn type_text(engine: &mut Engine, text: &str) {
        for character in text.chars() {
            engine.input(Input::Character(character));
        }
    }

    #[test]
    fn telex_vowels_and_tones() {
        let mut engine = engine();
        for (raw, rendered) in [
            ("aa", "â"),
            ("aw", "ă"),
            ("ee", "ê"),
            ("oo", "ô"),
            ("ow", "ơ"),
            ("uw", "ư"),
            ("as", "á"),
            ("af", "à"),
            ("ar", "ả"),
            ("ax", "ã"),
            ("aj", "ạ"),
            ("aas", "ấ"),
            ("aaf", "ầ"),
            ("aar", "ẩ"),
            ("aax", "ẫ"),
            ("aaj", "ậ"),
            ("aws", "ắ"),
            ("ows", "ớ"),
            ("uws", "ứ"),
        ] {
            engine.reset();
            type_text(&mut engine, raw);
            assert_eq!(engine.rendered(), rendered, "{raw}");
        }
        engine.reset();
        type_text(&mut engine, "w");
        assert_eq!(engine.rendered(), "ư");
    }

    #[test]
    fn vni_vowels_and_tones() {
        let mut engine = engine();
        engine.set_renderer(SimpleRenderer::new(
            SimpleInterpreter::vni(),
            Orthography::default(),
        ));
        type_text(&mut engine, "a61");
        assert_eq!(engine.rendered(), "ấ");
        engine.reset();
        type_text(&mut engine, "a71");
        assert_eq!(engine.rendered(), "ắ");
    }

    #[test]
    fn strict_rules_allow_valid_syllables() {
        let mut engine = engine();
        type_text(&mut engine, "ban");
        assert_eq!(engine.rendered(), "ban");

        engine.reset();
        type_text(&mut engine, "aas");
        assert_eq!(engine.rendered(), "ấ");

        engine.reset();
        type_text(&mut engine, "ows");
        assert_eq!(engine.rendered(), "ớ");

        engine.reset();
        type_text(&mut engine, "tieesng");
        assert_eq!(engine.rendered(), "tiếng");
    }

    #[test]
    fn strict_rules_reject_invalid_transformations() {
        let mut engine = engine();
        type_text(&mut engine, "aaw");
        assert_eq!(engine.rendered(), "aaw");

        engine.reset();
        type_text(&mut engine, "xxaa");
        assert_eq!(engine.rendered(), "xxaa");
    }

    #[test]
    fn tone_uses_the_main_vowel_of_a_vowel_sequence() {
        let mut engine = engine();
        type_text(&mut engine, "tieesng");
        assert_eq!(engine.rendered(), "tiếng");

        engine.reset();
        type_text(&mut engine, "hoas");
        assert_eq!(engine.rendered(), "hoá");
    }

    #[test]
    fn tone_insertion_uses_the_actual_cursor_position() {
        let mut engine = engine();
        type_text(&mut engine, "hoang");

        engine.input(Input::Home);
        for _ in 0..3 {
            engine.input(Input::Right);
        }
        engine.input(Input::Character('s'));
        assert_eq!(engine.composition().raw(), "hoasng");
        assert_eq!(engine.rendered(), "hoáng");

        engine.reset();
        type_text(&mut engine, "hoang");
        engine.input(Input::Home);
        engine.input(Input::Right);
        engine.input(Input::Character('s'));
        assert_eq!(engine.composition().raw(), "hsoang");
        assert_eq!(engine.rendered(), "hsoang");
    }

    #[test]
    fn backspace_rebuilds_from_raw_input() {
        let mut engine = engine();
        type_text(&mut engine, "aas");
        engine.input(Input::Backspace);
        assert_eq!(engine.composition().raw(), "aa");
        assert_eq!(engine.rendered(), "â");
    }

    #[test]
    fn space_commits_without_mutating_previous_text() {
        let mut engine = engine();
        type_text(&mut engine, "aas");
        let output = engine.input(Input::Space);
        assert_eq!(output, Result::Commit("ấ ".into()));
        type_text(&mut engine, "xin");
        assert_eq!(engine.rendered(), "xin");
    }

    #[test]
    fn identical_inputs_produce_identical_state() {
        let mut first = engine();
        let mut second = engine();
        type_text(&mut first, "aas");
        type_text(&mut second, "aas");
        assert_eq!(first.rendered(), second.rendered());
        assert_eq!(first.composition(), second.composition());
    }

    #[test]
    fn arbitrary_unicode_is_safe() {
        let mut engine = engine();
        for character in "日本語🙂e\u{301}".chars() {
            engine.input(Input::Character(character));
        }
        assert!(!engine.rendered().is_empty());
        assert!(engine.composition().cursor() <= engine.composition().raw().len());
    }

    #[test]
    fn canonical_raw_composes_precomposed_words() {
        let mut engine = engine();

        type_text(&mut engine, "nguowif");
        assert_eq!(engine.composition().raw(), "nguowif");
        assert_eq!(engine.rendered(), "người");

        engine.reset();
        for character in "người".chars() {
            engine.input(Input::Character(character));
        }
        assert_eq!(engine.composition().raw(), "nguowif");
        assert_eq!(engine.rendered(), "người");
    }

    #[test]
    fn duplicate_horn_markers_merge_into_canonical_raw() {
        let mut engine = engine();
        type_text(&mut engine, "nguwowif");
        assert_eq!(engine.composition().raw(), "nguowif");
        assert_eq!(engine.rendered(), "người");
    }

    #[test]
    fn trailing_tone_markers_are_relocated_before_the_coda() {
        let mut engine = engine();
        type_text(&mut engine, "vanhf");
        assert_eq!(engine.composition().raw(), "vanfnh");
        assert_eq!(engine.rendered(), "vành");

        engine.reset();
        type_text(&mut engine, "truongf");
        assert_eq!(engine.rendered(), "trường");
    }

    #[test]
    fn bare_w_and_vni_eight_compose_uhorn() {
        let mut engine = engine();
        type_text(&mut engine, "nhw");
        assert_eq!(engine.rendered(), "như");

        engine.reset();
        engine.set_renderer(SimpleRenderer::new(
            SimpleInterpreter::vni(),
            Orthography::default(),
        ));
        type_text(&mut engine, "u8");
        assert_eq!(engine.rendered(), "ư");
    }

    #[test]
    fn z_cancels_a_pending_tone() {
        let mut engine = engine();
        type_text(&mut engine, "aasz");
        assert_eq!(engine.composition().raw(), "aa");
        assert_eq!(engine.rendered(), "â");
    }

    #[test]
    fn common_words_round_trip_through_canonical_raw() {
        let mut engine = engine();
        for (typed, expected) in [
            ("vieetj", "việt"),
            ("quoocs", "quốc"),
            ("nguwx", "ngữ"),
            ("dduwowjc", "được"),
            ("xuaan", "xuân"),
            ("hoas", "hoá"),
        ] {
            engine.reset();
            type_text(&mut engine, typed);
            assert_eq!(engine.rendered(), expected, "{typed}");
        }
    }
}
