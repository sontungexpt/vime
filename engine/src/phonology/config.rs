/// Syllable grammar-checking mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ParsingMode {
    /// Enforce strict Vietnamese orthography.
    #[default]
    Strict,
    /// Allow teen code, English loanwords, and free typing (e.g. "ká", "kông", "quu").
    Loose,
}
